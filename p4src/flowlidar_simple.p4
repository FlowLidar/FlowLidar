/*
 * 
 * P4_16 for Tofino ASIC
 * Written May- 2022 for FlowLidar
 * 
 */
#include <core.p4>
#include <tna.p4>

#define ETHERTYPE_IPV4 0x0800
#define IP_PROTO_DISCO 0x33

typedef bit<32> ipv4_addr_t;
typedef bit<32> debug_t; //Size of debug info (sent as digest)
typedef bit<4> sketch_row_t;
typedef bit<32> sketch_hash_seed_t;
typedef bit<32> sketch_value_t;

//typedef bit<9> sketch_column_t; //can fit 512
typedef bit<15> sketch_column_t; //can fit 32,768
//typedef bit<17> sketch_column_t; //can fit 131,072

//#define NUM_SKETCH_COLS 512 //Dev default: 16 (ensure sketch_column_t is updated)
#define NUM_SKETCH_COLS 32768 //Minimizing staging: 32,768 
//#define NUM_SKETCH_COLS 131072 //Max size: 140K, minimize staging: 32,768 

typedef bit<17> bloomfilter_column_t; //can fit 131,072

#define NUM_BF_COLS 131072

header ethernet_h
{
	bit<48> dstAddr;
	bit<48> srcAddr;
	bit<16> etherType;
}

header ipv4_h
{
	bit<4> version;
	bit<4> ihl;
	bit<6> dscp;
	bit<2> ecn;
	bit<16> totalLen;
	bit<16> identification;
	bit<3> flags;
	bit<13> fragOffset;
	bit<8> ttl;
	bit<8> protocol;
	bit<16> hdrChecksum;
	bit<32> srcAddr;
	bit<32> dstAddr;
}

struct headers
{
	ethernet_h ethernet;
	ipv4_h ipv4;
}

struct cpu_digest_t
{
	debug_t debug;
	bit<32> srcAddr;
}

struct ingress_metadata_t
{
	debug_t debug;
	bit<1> send_cpu_digest;
	bit<8> bloom_filter_presketch_hit;
	bit<8> bloom_filter_predigest_hit;
	
	bit<8> bf_bit_0;
	bit<8> bf_bit_1;
	bit<8> bf_bit_2;
	bit<8> bf_bit_3;
	bit<8> bf_bit_4;
	
	sketch_value_t cms_output_0;
	sketch_value_t cms_output_1;
	sketch_value_t cms_output_2;
	sketch_value_t cms_output_3;
	sketch_value_t cms_output_4;
}

struct egress_metadata_t
{
	
}

parser TofinoIngressParser(packet_in pkt, out ingress_intrinsic_metadata_t ig_intr_md)
{
	state start
	{
		pkt.extract(ig_intr_md);
		transition select(ig_intr_md.resubmit_flag)
		{
			1 : parse_resubmit;
			0 : parse_port_metadata;
		}
	}

	state parse_resubmit
	{
		transition reject;
	}

	state parse_port_metadata
	{
		pkt.advance(64); //Tofino 1
		transition accept;
	}
}

parser SwitchIngressParser(packet_in pkt, out headers hdr, out ingress_metadata_t ig_md, out ingress_intrinsic_metadata_t ig_intr_md)
{
	TofinoIngressParser() tofino_parser;
	
	state start 
	{
		tofino_parser.apply(pkt, ig_intr_md);
		transition parse_ethernet;
	}
	
	state parse_ethernet
	{
		pkt.extract(hdr.ethernet);
		transition select(hdr.ethernet.etherType)
		{
			ETHERTYPE_IPV4: parse_ipv4;
			default: accept;
		}
	}
	
	state parse_ipv4
	{
		pkt.extract(hdr.ipv4);
		transition accept;
	}
	
	
}

control ControlCMS_row(inout headers hdr, inout ingress_metadata_t ig_md, inout sketch_value_t md_output)(bit<32> hash_seed)
{
	//Use custom CRC polynomial
	CRCPolynomial<bit<32>>(hash_seed, true, false, false, 32w0xFFFFFFFF, 32w0xFFFFFFFF) poly1;                               
	Hash<sketch_column_t>(HashAlgorithm_t.CUSTOM, poly1) hash_sketch_column;
	
	
	Register<sketch_value_t, sketch_column_t>(NUM_SKETCH_COLS,0) reg_row_columns;
	RegisterAction<sketch_value_t, sketch_column_t, sketch_value_t>(reg_row_columns) sketch_count = {
		void apply(inout sketch_value_t value, out sketch_value_t output)
		{
			//TODO: build in rollover protection
			value = value + 1; //Increment counter
			output = value;
		}
	};
	
	
	apply
	{
		md_output = sketch_count.execute(hash_sketch_column.get({hdr.ipv4.srcAddr}));
	}
}

control ControlCMS(inout headers hdr, inout ingress_metadata_t ig_md)
{
	//Create Control blocks for each row (unique registers, unique seeds)
	ControlCMS_row(0x1e12a700) cms_row_0;
	ControlCMS_row(0x65b96595) cms_row_1;
	ControlCMS_row(0x49cf878b) cms_row_2;
	ControlCMS_row(0x36518f0d) cms_row_3;
	ControlCMS_row(0x7a40a908) cms_row_4;
	
	
	apply
	{
		//Specify which sketch rows to apply
		cms_row_0.apply(hdr, ig_md, ig_md.cms_output_0);
		cms_row_1.apply(hdr, ig_md, ig_md.cms_output_1);
		cms_row_2.apply(hdr, ig_md, ig_md.cms_output_2);
		cms_row_3.apply(hdr, ig_md, ig_md.cms_output_3);
		cms_row_4.apply(hdr, ig_md, ig_md.cms_output_4);
	}
}

//TODO: make this into BF row
control ControlBloomFilter_row(inout headers hdr, inout ingress_metadata_t ig_md, inout bit<8> md_result)(bit<32> hash_seed)
{
	//Use custom CRC polynomial
	CRCPolynomial<bit<32>>(hash_seed, true, false, false, 32w0xFFFFFFFF, 32w0xFFFFFFFF) poly1;                               
	Hash<bloomfilter_column_t>(HashAlgorithm_t.CUSTOM, poly1) hash_bloomfilter_index;
	
	
	Register<bit<8>, bloomfilter_column_t>(NUM_BF_COLS,0) reg_row_columns;
	RegisterAction<bit<8>, bloomfilter_column_t, bit<8>>(reg_row_columns) row_read_and_set = {
		void apply(inout bit<8> value, out bit<8> output)
		{
			output = value;
			value = 1;
		}
	};
	
	
	apply
	{
		md_result = row_read_and_set.execute(hash_bloomfilter_index.get({hdr.ipv4.srcAddr}));
	}
}

control ControlBloomFilter(inout headers hdr, inout ingress_metadata_t ig_md, inout bit<8> md_is_hit)
{
	//Polynomials are purely random 2^31->2^32
	ControlBloomFilter_row(0xd1de7aab) bf_row_0;
	ControlBloomFilter_row(0x81e9b7a4) bf_row_1;
	ControlBloomFilter_row(0xe9fbae42) bf_row_2;
	ControlBloomFilter_row(0x937d2ad6) bf_row_3;
	
	action set_bf_miss()
	{
		md_is_hit = 0;
	}
	action set_bf_hit()
	{
		md_is_hit = 1;
	}
	table tbl_detect_bf_miss
	{
		key = {
			ig_md.bf_bit_0: exact;
			ig_md.bf_bit_1: exact;
			ig_md.bf_bit_2: exact;
			ig_md.bf_bit_3: exact;
		}
		actions = {
			set_bf_miss;
			set_bf_hit;
		}
		const default_action = set_bf_miss;
		const entries = {
			(1,1,1,1): set_bf_hit();
		}
		size=10;
	}
	
	apply
	{
		bf_row_0.apply(hdr, ig_md, ig_md.bf_bit_0);
		bf_row_1.apply(hdr, ig_md, ig_md.bf_bit_1);
		bf_row_2.apply(hdr, ig_md, ig_md.bf_bit_2);
		bf_row_3.apply(hdr, ig_md, ig_md.bf_bit_3);
		
		tbl_detect_bf_miss.apply();
	}
}

control ControlFlowLidar(inout headers hdr, inout ingress_metadata_t ig_md)
{
	ControlCMS() cms;
	ControlBloomFilter() bloom_filter_presketch;
	ControlBloomFilter() bloom_filter_predigest;
	apply
	{
		bloom_filter_presketch.apply(hdr, ig_md, ig_md.bloom_filter_presketch_hit);
		
		if( ig_md.bloom_filter_presketch_hit == 1) //Only apply sketch if first BF is hit (to filter out single-packet flows)
		{
			bloom_filter_predigest.apply(hdr, ig_md, ig_md.bloom_filter_predigest_hit);
			cms.apply(hdr, ig_md);
			
			if( ig_md.bloom_filter_predigest_hit == 0) //Only send digest if flow is counted in the sketch, and then again only once per flow
			{
				ig_md.send_cpu_digest = 1;
			}
		}
	}
}

control SwitchIngress(inout headers hdr, inout ingress_metadata_t ig_md, in ingress_intrinsic_metadata_t ig_intr_md, in ingress_intrinsic_metadata_from_parser_t ig_intr_prsr_md, inout ingress_intrinsic_metadata_for_deparser_t ig_intr_dprsr_md, inout ingress_intrinsic_metadata_for_tm_t ig_intr_tm_md)
{
	ControlFlowLidar() FlowLidar;
	
	action forward(PortId_t port)
	{
		ig_intr_tm_md.ucast_egress_port = port; //Set egress port
		hdr.ipv4.ttl = hdr.ipv4.ttl - 1;
	}
	action drop()
	{
		ig_intr_dprsr_md.drop_ctl = 1;
	}
	table tbl_forward
	{
		key = {
			hdr.ipv4.dstAddr: exact;
		}
		actions = {
			forward;
			@defaultonly drop;
		}
		default_action = drop;
		size=256;
	}

	apply
	{
		ig_md.send_cpu_digest = 0; //Default, unless it flips in FlowLidar control block
		if(hdr.ipv4.isValid())
		{
			tbl_forward.apply();
			//Run FlowLidar
			FlowLidar.apply(hdr, ig_md);
		}
	}
}

control SwitchIngressDeparser(packet_out pkt, inout headers hdr, in ingress_metadata_t ig_md, in ingress_intrinsic_metadata_for_deparser_t ig_intr_dprsr_md)
{
	Digest<cpu_digest_t>() cpu_digest;
	Checksum() ipv4_checksum;
	
	apply
	{
		//Update IPv4 checksum
		hdr.ipv4.hdrChecksum = ipv4_checksum.update(
			{hdr.ipv4.version,
			 hdr.ipv4.ihl,
			 hdr.ipv4.dscp,
			 hdr.ipv4.ecn,
			 hdr.ipv4.totalLen,
			 hdr.ipv4.identification,
			 hdr.ipv4.flags,
			 hdr.ipv4.fragOffset,
			 hdr.ipv4.ttl,
			 hdr.ipv4.protocol,
			 hdr.ipv4.srcAddr,
			 hdr.ipv4.dstAddr});
		
		//Compile CPU digest
		if( ig_md.send_cpu_digest == 1 )
		{
			cpu_digest.pack({
				ig_md.debug,
				hdr.ipv4.srcAddr
			});
		}
		
		pkt.emit(hdr);
	}
}

parser TofinoEgressParser(packet_in pkt, out egress_intrinsic_metadata_t eg_intr_md)
{
	state start
	{
		pkt.extract(eg_intr_md);
		transition accept;
	}
}

parser SwitchEgressParser(packet_in pkt, out headers hdr, out egress_metadata_t eg_md, out egress_intrinsic_metadata_t eg_intr_md)
{
	TofinoEgressParser() tofino_parser;

	state start
	{
		tofino_parser.apply(pkt, eg_intr_md);
		transition accept;
	}
}

control SwitchEgress(inout headers hdr, inout egress_metadata_t eg_md, in egress_intrinsic_metadata_t eg_intr_md, in egress_intrinsic_metadata_from_parser_t eg_intr_from_prsr, inout egress_intrinsic_metadata_for_deparser_t eg_intr_md_for_dprsr, inout egress_intrinsic_metadata_for_output_port_t eg_intr_md_for_oport)
{
	apply
	{
		
	}
}

control SwitchEgressDeparser(packet_out pkt, inout headers hdr, in egress_metadata_t eg_md, in egress_intrinsic_metadata_for_deparser_t eg_dprsr_md)
{
	apply
	{
		pkt.emit(hdr);
	}
}


Pipeline(SwitchIngressParser(),
	SwitchIngress(),
	SwitchIngressDeparser(),
	SwitchEgressParser(),
	SwitchEgress(),
	SwitchEgressDeparser()
) pipe;

Switch(pipe) main;
