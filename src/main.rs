extern crate nalgebra as na;
use na::{DMatrix};
use FlowLidar::bloom_filter;
use FlowLidar::cms;
use FlowLidar::approx::*;
use std::ops::Mul;

pub use pcap_parser::traits::PcapReaderIterator;
pub use pcap_parser::*;
use pcap_parser::data::{get_packetdata, PacketData};
pub use std::fs::File;
pub use std::io::ErrorKind;
pub use std::io::Read;
pub use packet::ether::Packet as EthernetPacket; 
pub use packet::ip::Packet as IpPacket;
pub use packet::tcp::Packet as TcpPacket;
pub use packet::udp::Packet as UdpPacket;
pub use packet::Packet;
pub use csv::Writer;
pub use std::io::Write;
use clap::{Arg, Command};
use std::process;


fn main() {
    let args: Vec<String> = std::env::args().collect();

    println!("command line is: {:?}", &args);
    let command_line = Command::new("FlowLidar simulator")
        .version("0.1.0")
        .author("sp")
        .about("Simulate FLowLidar monitoring system")
        .arg(Arg::new("filename")
             .short('f')
             .long("file")
             .takes_value(true)
             .default_value("./test.pcap")
             .help("pcap file"))
        .arg(Arg::new("sip")
             .long("sip")
             .takes_value(false)
             //.default_value(false)
             .help("use sip as flow key"))
        .arg(Arg::new("approx")
             .short('a')
             .long("approx")
             .takes_value(false)
             //.default_value(false)
             .help("use approximate resolution"))
        .arg(Arg::new("lazy")
             .short('l')
             .long("lazy")
             .conflicts_with("old_new")
             .takes_value(false)
             //.default_value(false)
             .help("use lazy BF"))
        .arg(Arg::new("old_new")
             .short('o')
             .long("old_new")
             .takes_value(false)
             .conflicts_with("lazy")
             //.default_value(false)
             .help("use old new pair of BFs"))
        .arg(Arg::new("skip")
             //.short('s')
             .long("skip")
             .takes_value(false)
             //.default_value(false)
             .help("skip AX=b solver"))
        .arg(Arg::new("stop")
             .long("stop")
             .takes_value(true)
             .default_value("0")
             //.default_value(false)
             .help("stop after n epochs"))
        .arg(Arg::new("k_bloom_filter")
             .short('k')
             .long("kbf")
             .takes_value(true)
             .default_value("4")
             .help("hash functions for the Bloom Filter"))
        .arg(Arg::new("bf_size")
             .short('s')
             .long("bf_size")
             .takes_value(true)
             .default_value("524288") //512Kb
             .help("Bloom Filter size"))
        .arg(Arg::new("num_cms")
             .short('n')
             .long("num_cms")
             .takes_value(true)
             .default_value("64")
             .help("num of CMS"))
        .arg(Arg::new("cms_size")
             .short('S')
             .long("cms_size")
             .takes_value(true)
             .default_value("256")
             .help("size of a CMS row"))
        .arg(Arg::new("kcms")
             .short('K')
             .long("kcms")
             .takes_value(true)
             .default_value("4")
             .help("hash functions for the CMS"))
        .arg(Arg::new("epoch_time")
             .short('e')
             .long("epoch")
             .takes_value(true)
             .default_value("1.0")
             .help("time between epochs"));

    let matches = command_line.clone().get_matches();
    println!("parameters are:");
    for a in command_line.get_arguments() {
        if a.is_takes_value_set() {
            println!("{:?} --> {:?}",a.get_id(),matches.value_of(a.get_id()).unwrap_or("None"));
        }
        else {
            println!("{:?} --> {:?}",a.get_id(),matches.is_present(a.get_id()));
        }
    }
    
    let filename = matches.value_of("filename").unwrap();
    let approx = matches.is_present("approx");
    let lazy = matches.is_present("lazy");
    let old_new = matches.is_present("old_new");
    let skip= matches.is_present("skip");
    let sip= matches.is_present("sip");
    let epoch_time=matches.value_of("epoch_time").unwrap().parse::<f64>().unwrap();
    let stop = matches.value_of("stop").unwrap().parse::<u32>().unwrap();
    let kbf=matches.value_of("k_bloom_filter").unwrap().parse::<usize>().unwrap();
    let bf_size=matches.value_of("bf_size").unwrap().parse::<usize>().unwrap();
    let num_cms=matches.value_of("num_cms").unwrap().parse::<usize>().unwrap();
    let cms_size=matches.value_of("cms_size").unwrap().parse::<usize>().unwrap();
    let kcms=matches.value_of("kcms").unwrap().parse::<usize>().unwrap();



    let mut num_items_per_block = vec![0; num_cms];

    let mut bf = bloom_filter::BloomFilter::build_bloom_filter(bf_size,kbf);
    let mut old_bf = bloom_filter::BloomFilter::build_bloom_filter(bf_size,kbf);
    let mut cms = cms::CMS::build_cms(cms_size,kcms,num_cms);
    println!("Overall CMS size is {} bits.",num_cms*kcms*cms_size*16);
    //let mut epoch_time=0.1; //4x256x16 50min
    //let mut epoch_time=1.0; //4x1024x16

    let mut if_linktypes = Vec::new();
    let mut trace_linktype;
    let mut file = File::open(filename).unwrap();
    let mut buffer = Vec::new();
    let mut hashmap = std::collections::HashMap::new();
    let mut old_hashmap = std::collections::HashMap::new();
    let mut controller_hashmap = std::collections::HashMap::new();
    let mut old_controller_hashmap = std::collections::HashMap::new();
    let mut removed_controller_hashmap = std::collections::HashMap::new();
    let mut first_packet=true;
    let mut epoch=0;
    let mut t0=0.0;
    let mut num_packets = 0;
    let mut send_to_controller = 0.0;
    let mut fp=0;
    let mut aae=0;
    let mut mae=0;
    let mut are=0.0;
    let mut mre:f32=0.0;

    println!("stat:\tEpoch\tnum_flows\tFP\tSolved\tExact\tNear_Exact\tAAE\tARE\tBW\tCMS_with_min_AAE\tCMS_with_min_ARE");
    
    file.read_to_end(&mut buffer).unwrap();
    // try pcap first
    match PcapCapture::from_file(&buffer) {
        Ok(capture) => {
            println!("Format: PCAP");
            //setting PCAP packet type
            trace_linktype = capture.header.network;
            for block in capture.iter() {
                match block {
                    PcapBlock::LegacyHeader(packet_header) => {
                        println!("Read pcap header!");
                        println!("{:?}", packet_header);
                        trace_linktype = packet_header.network;
                    }
                    PcapBlock::NG(Block::SectionHeader(ref _shb)) => {
                        // starting a new section, clear known interfaces
                        if_linktypes = Vec::new();
                        println!("ng block header");
                    }
                    PcapBlock::NG(Block::InterfaceDescription(ref idb)) => {
                        if_linktypes.push(idb.linktype);
                        println!("ng block interface desc");
                    }
                    PcapBlock::NG(Block::EnhancedPacket(ref epb)) => {
                        assert!((epb.if_id as usize) < if_linktypes.len());
                        println!("ng block enh pack");
                        #[cfg(feature = "data")]
                        let res = pcap_parser::data::get_packetdata(
                            epb.data,
                            linktype,
                            epb.caplen as usize,
                        );
                    }
                    PcapBlock::NG(Block::SimplePacket(ref _spb)) => {
                        assert!(if_linktypes.len() > 0);
                        println!("ng block simple pack");
                        #[cfg(feature = "data")]
                        let res = pcap_parser::data::get_packetdata(spb.data, linktype, blen);
                    }
                    PcapBlock::NG(_) => {
                        // can be statistics (ISB), name resolution (NRB), etc.
                        println!("ng block unsup");
                        eprintln!("unsupported block");
                    }

                    PcapBlock::Legacy(packet) => {
                        let pkt_data = get_packetdata(packet.data, trace_linktype, packet.caplen as usize).unwrap();
                        //println!("usec {}",packet.ts_sec as f64 + packet.ts_usec as f64 / 1000000.0);
                        let mut ts = packet.ts_sec as f64 + (packet.ts_usec as f64 /1000000.0);
                        let l2_packet; 
                        let l3_packet;
                        let l4_tcp_packet;
                        let l4_udp_packet;
                        let proto;
                        let src_port;
                        let dst_port;
                        
                        //println!("read packet");
                        match pkt_data {
                            PacketData::L2(a) => {
                                //println!("Ethernet packet");
                                l2_packet = EthernetPacket::new(a).unwrap();
                                //unchecked as there's no payload
                                if l2_packet.protocol() != packet::ether::Protocol::Ipv4 {
                                    continue;
                                }
                                let temp_l3 = IpPacket::unchecked(l2_packet.payload());
                                match temp_l3 {
                                    IpPacket::V4(p) => {
                                        l3_packet = p;
                                    },
                                    _ => {   continue; }
                                }
                                if l3_packet.protocol() == packet::ip::Protocol::Tcp {
                                    proto=0x06;
                                    //println!("tcp inside ip");
                                    //println!("l3_payload: {:?}",l3_packet.payload());
                                    l4_tcp_packet = TcpPacket::new(l3_packet.payload()).unwrap();
                                    src_port = l4_tcp_packet.source();
                                    dst_port = l4_tcp_packet.destination();
                                    //println!("{:?}", l4_tcp_packet);
                                } 
                                else {
                                    if l3_packet.protocol() == packet::ip::Protocol::Udp {
                                        proto=0x11;
                                        //println!("udp inside ip");
                                        //src_port = l4_udp_packet.source();
                                        //dst_port = l4_udp_packet.destination();
                                        //l4_ucp_packet = UdpPacket::new(l3_packet.payload()).unwrap();
                                        let res=UdpPacket::new(l3_packet.payload()); 
                                        match res {
                                            Ok(l4_udp_packet) => {
                                                src_port = l4_udp_packet.source();
                                                dst_port = l4_udp_packet.destination();
                                            },
                                            Err(_) => {
                                                if l3_packet.payload().len()<4 {continue;} 
                                                src_port = 256*(l3_packet.payload()[0] as u16) + l3_packet.payload()[1] as u16;
                                                dst_port = 256*(l3_packet.payload()[2] as u16) + l3_packet.payload()[3] as u16;
                                            }
                                        }
                                    }
                                    else {                                    
                                        //println!("not tcp/udp");
                                        continue;
                                    }
                                }
                            },
                            PacketData::L3(_, b) => {
                                let temp_l3 = IpPacket::unchecked(b);
                                match temp_l3 {
                                    IpPacket::V4(p) => {l3_packet = p; },
                                    _ => { continue; }

                                }
                                if l3_packet.protocol() == packet::ip::Protocol::Tcp {
                                    //println!("tcp inside ip");
                                    proto=0x06;
                                    match TcpPacket::new(l3_packet.payload()) {
                                        Ok(p) => l4_tcp_packet = p,
                                        _ => continue,
                                    }
                                    src_port = l4_tcp_packet.source();
                                    dst_port = l4_tcp_packet.destination();
                                    //println!("{:?}", l4_tcp_packet);
                                } else {
                                    if l3_packet.protocol() == packet::ip::Protocol::Udp {
                                        proto=0x11;
                                        match UdpPacket::new(l3_packet.payload()) {
                                            Ok(p) => l4_udp_packet = p,
                                            _ => continue,
                                        }
                                        src_port = l4_udp_packet.source();
                                        dst_port = l4_udp_packet.destination();
                                    }
                                    else {                                    
                                        //println!("not tcp/udp");
                                        continue;
                                    }
                                }
                            },
                                    
                            PacketData::L4(_, _) => {
                                println!("L4 type");
                                continue;
                            },
                            PacketData::Unsupported(_a) => {
                                println!("Unsupported");
                                continue;
                            },
                        }
/**************************************************
*  Packet processing starts here
**************************************************/


                        let key  = if sip {
                                (l3_packet.source(), l3_packet.source(), 0, 0, 0)
                            }
                            else {
                                (l3_packet.source(), l3_packet.destination(), proto, src_port, dst_port)
                            }
                        ;
                        if first_packet {
                            t0=ts; 
                            first_packet=false;
                            println!("new epoch: [{}] ", epoch);
                        }
                        ts = ts-t0-epoch_time*(epoch as f64);
                        if ts>epoch_time {
                            epoch +=1;
                            if epoch==stop {
                                process::exit(1);
                            }
                            //ts -=epoch_time;

                            //end of epoch: collect results
                            println!("#packets {}", num_packets);
                            println!("#packets to the controller {}", send_to_controller);
                            println!("#flows {}", hashmap.len());
                            println!("#flows in the Control Plane {}", controller_hashmap.len());
                            println!("#items in the bf {}", bf.get_num_items());
                            println!("#items in the map {}", hashmap.len());
                            println!("load in the bf {}", bf.get_load());
                            println!("load in the CMS {}", cms.get_load());
                            
                            let mut negative_on_cp=0;
                            if lazy { 
                                // remove flows not in the BF
                                let mut pruned_controller_hashmap= controller_hashmap.clone();
                                for (k,v) in &controller_hashmap {
                                    if !bf.query(k) {
                                        negative_on_cp +=1;
                                        pruned_controller_hashmap.remove(k);
                                        removed_controller_hashmap.insert(*k,*v);
                                    }
                                }
                                println!("#flows in the Control Plane removed by lazy BF: {} [{}%]", negative_on_cp, (100*negative_on_cp)/controller_hashmap.len());
                                controller_hashmap= pruned_controller_hashmap;
                            }
                            let mut added=0.0;
                            if old_new {
                                for (k,_) in &old_controller_hashmap { 
                                    if bf.query(k) {  //old flows are also in the new epoch
                                        controller_hashmap.insert(*k,1); 
                                        added +=1.0;
                                    }
                                }
                                println!("#flows in the Control Plane added from the past: {}",added);
                                //pruned_controller_hashmap= controller_hashmap.clone();
                            }

                            //recompute num_items_per_block[]
                            for i in 0..num_cms {
                                num_items_per_block[i] = 0;
                            }
                            for (k,_) in &controller_hashmap {
                                let (b,_)=cms.index(k);
                                num_items_per_block[b] +=1;  
                            }
                            //create the array of Ax=b linear system
                            let mut tot_solved=0.0;
                            let mut tot_num_exact=0.0;
                            let mut tot_num_near_exact=0.0;
                            let mut tot_error=0.0;
                            let mut tot_relative_error=0.0;
                            let mut min_load:f32=1.0;
                            let mut max_load:f32=0.0;
                            if !skip {
                                let mut handles = Vec::new();
                                for block_iter in 0..num_cms { 
                                    let num_items_per_this_block=num_items_per_block[block_iter];
                                    let hashmap=hashmap.clone();
                                    let controller_hashmap=controller_hashmap.clone();
                                    let removed_controller_hashmap=removed_controller_hashmap.clone();
                                    let cms=cms.clone();
                                    //let handle = thread::spawn(move || {
                                    let handle = { 
                                        println!("create the {}-th Ax=b linear system",block_iter);
                                        let mut matrix_a= DMatrix::zeros(kcms*cms_size,num_items_per_this_block);
                                        let mut vector_b= DMatrix::zeros(kcms*cms_size,1);
                                        let mut num_exact=0.0;
                                        let mut num_near_exact=0.0;
                                        let mut solved=0.0;
                                        let mut error=0.0;
                                        let mut relative_error=0.0;
                                        let mut j=0;
                                        for (k,_) in &controller_hashmap {
                                            let idx=cms.index(k);
                                            if idx.0 !=block_iter {
                                                continue;
                                            }
                                            for i in idx.1 {
                                                matrix_a[(i as usize,j)]=1.0 as f32;
                                            }
                                            j +=1;
                                        }
                                        for i in 0..kcms*cms_size {
                                            vector_b[(i,0)]=cms.filter_bins[block_iter*kcms*cms_size+i] as f32;
                                        }
                                        // try to solve!
                                        // see: https://gitlab.com/rust-qr-factorization/linalgrs/-/tree/master/
                                        let load_system = matrix_a.ncols() as f32/ matrix_a.nrows() as f32;
                                        println!("solve the {}-th Ax=b linear system with A[{}x{}] ({:})",block_iter,matrix_a.nrows(),matrix_a.ncols(),load_system);
                                        let mut solution_x = DMatrix::zeros(matrix_a.ncols(),1);
                                        for j in 0..matrix_a.ncols() {
                                            solution_x[(j,0)]=1000000.0; //solution_x[(j,0)]) will be bounded by standard CMS
                                        }
                                        if matrix_a.nrows()> matrix_a.ncols() { // can be solved exactly
                                            let decomp = matrix_a.qr();
                                            //let q = decomp.q();
                                            let qt = decomp.q().transpose();
                                            let r = decomp.unpack_r();
                                            //let sol = decomp.solve(&vector_b);
                                            
                                            match r.clone().try_inverse() {
                                                None => {},
                                                Some(rm1) => {
                                                    let b1 = qt.mul(&vector_b);
                                                    solution_x=rm1.mul(&b1);
                                                    solved +=1.0;
                                                },
                                            }
                                        }
                                        //cannot be solved directly:
                                        else {
                                             if approx { 
                                                 solution_x=approx_solve_qr(matrix_a,vector_b);
                                             }
                                        }

                                        //compare with actual values
                                        println!("{}-th system: compare with actual values",block_iter);

                                        // count the real number of flows for the i-th CMS and dump the
                                        // flow values not in the CMS
                                        let mut real_number_of_flows=0;
                                        let mut untracked_flows=0;
                                        let mut removed_flows=0;
                                        let mut j=0;
                                        for (k,_) in &controller_hashmap { // solved by Ax=b 
                                            let idx=cms.index(k);
                                            if idx.0 !=block_iter {
                                                continue;
                                            }
                                            let mut s;
                                            real_number_of_flows +=1;
                                            if lazy {
                                                let offset = controller_hashmap.get(k).unwrap(); 
                                                //s= (*offset as f32 + solution_x[(j,0)]).round();
                                                let value = (solution_x[(j,0)]).round().min((cms.query(k)+1+kbf as u32) as f32);
                                                s= *offset as f32 + value;
                                            }
                                            else { 
                                                s= (1.0+solution_x[(j,0)]).round();
                                                if !old_new {    
                                                    s= s.max(1.0);
                                                }
                                                s = s.min((cms.query(k)+1) as f32); //bound with default CMS
                                            }
                                            j +=1;
                                            let exact= *hashmap.get(k).unwrap_or(&1.0) as f32; //use .unwrap_or for old_new (is not in the hashmap)
                                            let e= (exact - s).abs();
                                            error +=e;
                                            relative_error +=e/exact;
                                            //if e>=1.0 {
                                            //    println!("variable {} --> {}=={} ({}) ",j,s,hashmap.get(k).unwrap(),e);
                                            //}
                                            if e<1.0 {
                                                num_exact +=1.0;
                                            }
                                            if e<2.0 {
                                                num_near_exact +=1.0;
                                            }
                                        }
                                        for (k,_) in &removed_controller_hashmap { // removed by lazy BF 
                                            let idx=cms.index(k);
                                            if idx.0 !=block_iter {
                                                continue;
                                            }
                                            real_number_of_flows +=1;
                                            let offset = removed_controller_hashmap.get(k).unwrap(); 
                                            let s= (*offset as f32).round();
                                            removed_flows +=1;
                                            let exact= *hashmap.get(k).unwrap() as f32;
                                            let e= (exact - s).abs();
                                            error +=e;
                                            relative_error +=e/exact;
                                            //if e>=1.0 {
                                            //    println!("variable {} --> {}=={} ({}) ",j,s,hashmap.get(k).unwrap(),e);
                                            //}
                                            if e<1.0 {
                                                num_exact +=1.0;
                                            }
                                            if e<2.0 {
                                                num_near_exact +=1.0;
                                            }
                                            //println!("removed flow {:?} --> {}=={} ({})",k,hashmap.get(k).unwrap(),s,e);
                                        }
                                        for (k,v) in &hashmap { // False positives 
                                            let idx=cms.index(k);
                                            if idx.0 !=block_iter {
                                                continue;
                                            }
                                            if controller_hashmap.get(k)==None && removed_controller_hashmap.get(k)==None { // untracked flow 
                                                real_number_of_flows +=1;
                                                untracked_flows +=1;
                                                error +=*v as f32;
                                                relative_error +=1.0;
                                                //println!("untracked flow {} --> {} ",untracked_flows, hashmap.get(k).unwrap());
                                            }
                                        }
                                        println!("{}-th system: num flow with exact count: {}. near exact: {}. Flows in the CMS {}. Real number of flows  {}. Untracked flows {}. Removed by lazy BF {}.",block_iter,num_exact,num_near_exact,num_items_per_this_block,real_number_of_flows,untracked_flows,removed_flows);
                                        //return partial results
                                        (num_exact,num_near_exact,error,relative_error,solved,load_system)
                                        //process::exit(1);
                                    }; 
                                    //);
                                    handles.push(handle);
                                }
                                for handle in handles {
                                            //let res=handle.join().unwrap();
                                            let res=handle;
                                            tot_num_exact += res.0;
                                            tot_num_near_exact += res.1;
                                            tot_error += res.2;
                                            tot_relative_error += res.3;
                                            tot_solved += res.4;
                                            min_load = min_load.min(res.5);
                                            max_load = max_load.max(res.5);
                                }
                            }
                            //}
                            //process::exit(1);

                            //traditional CMS
                            for (k,v) in &hashmap {
                                //println!("TRUE k: {:?} v: {}",k,v);
                                //println!("CMS  k: {:?} v: {}",k,cms.query(k));
                                let e = ((cms.query(k)+1-*v as u32) as i32).abs();
                                aae += e; 
                                mae = mae.max(e);
                                let re = (e as f32)/(*v as f32);
                                are += re; 
                                mre = mre.max(re);
                            }
                            let num_flows = hashmap.len()as f32;
                            let average_ae= (aae as f32)/num_flows;
                            let average_re= (are as f32)/num_flows;
                            println!("AAE {:.2}", average_ae);
                            println!("MAE {}", mae);
                            println!("ARE {:.0}%", 100.0*are/num_flows);
                            println!("MRE {}%", 100.0*mre);
                            println!("num_flows: {}",num_flows);
                            println!("flows in CP {}",controller_hashmap.len());
                            println!("min max load: {} {}",min_load,max_load);
                            let num_fp:f32;
                            if lazy {
                                num_fp =num_flows - (negative_on_cp as f32) - (controller_hashmap.len() as f32);
                            }
                            else {
                                num_fp =fp as f32; //num_flows - controller_num_flows as f32;
                            }
                            println!("Matrix shape m: {} n: {} l:{}",num_cms*cms_size*kcms,hashmap.len(), num_flows/ (num_cms*cms_size*kcms) as f32);
                            //println!("stat:\tEpoch\tnum_flows\tFP\tSolved\tExact\tNear_Exact\tAAE\tARE\tBW\tCMS_with_min_AAE\tCMS_with_min_ARE");
                            println!("stat:\t{}\t{}\t{:.4}\t{:.2}\t{:.2}\t{:.2}\t{:.5}\t{:.5}\t{:.2}\t{:.2}\t{:.2}",epoch,num_flows,100.0*num_fp/num_flows,tot_solved/(num_cms as f32),100.0*tot_num_exact/num_flows,100.0*tot_num_near_exact/num_flows,tot_error/num_flows,tot_relative_error/num_flows,send_to_controller/epoch_time,average_ae,average_re);

                            
                            let mut histogram = std::collections::HashMap::new();
                            for (_,v) in &hashmap {
                                let counter = histogram.entry((*v as f32).round() as u32).or_insert(0);
                                *counter +=1; 
                            }
                            println!("#flows with 1 packet: {:?} {:.2}",histogram.get(&1).unwrap_or(&0),*histogram.get(&1).unwrap_or(&0) as f32/num_flows);
                            println!("#flows with 2 packet: {:?} {:.2}",histogram.get(&2).unwrap_or(&0),*histogram.get(&2).unwrap_or(&0) as f32/num_flows);
                            println!("#flows with 3 packet: {:?} {:.2}",histogram.get(&3).unwrap_or(&0),*histogram.get(&3).unwrap_or(&0) as f32/num_flows);
                            println!("#flows with 4 packet: {:?} {:.2}",histogram.get(&4).unwrap_or(&0),*histogram.get(&4).unwrap_or(&0) as f32/num_flows);
                            println!("#flows {:?}",hashmap.len());
                            //let mut count_vec: Vec<(_,_)> = hashmap.iter().collect();
                            //count_vec.sort_by(|a, b| b.1.partial_cmp(a.1).unwrap());
                            //println!("Most frequent key is : {:?} ", count_vec[0]);

                            // clear all for a new epoch
                            //process::exit(1);
                            println!("new epoch: [{}] ", epoch);
                            aae=0;
                            mae=0;
                            are=0.0;
                            mre=0.0;
                            fp=0;
                            if old_new {
                                old_bf=bf.clone();
                                old_controller_hashmap=controller_hashmap.clone();
                                old_hashmap=hashmap.clone();
                            }
                            hashmap.clear();
                            controller_hashmap.clear();
                            removed_controller_hashmap.clear();
                            bf.clear();
                            cms.clear();
                            num_packets =0;
                            send_to_controller =0.0;
                        }
                        num_packets += 1;
                        let counter = hashmap.entry(key).or_insert(0.0);
                        if *counter==0.0  && !old_new && bf.query(key) {
                               fp +=1;
                        }

                        if old_new && *counter==0.0  && ( bf.query(key) || (!old_hashmap.contains_key(&key) && old_bf.query(key)) ) {
                               fp +=1;
                        }
                        *counter +=1.0; 

                        
                       //1. insert flow in CMS if bf==true (skip 1-packet flows for BF and up to k
                       //   packets for the lazy BF)
                       if bf.query(key) {
                           cms.insert(key);
                       }
                       else {
                           if !old_new || !old_bf.query(key) {
                               send_to_controller +=1.0;
                               let counter = controller_hashmap.entry(key).or_insert(0);
                               *counter +=1;
                           }
                       }
                       //2. insert new flow in standard/lazy bf
                       if lazy {
                            bf.lazy_insert(key);
                       }
                       else {
                            bf.insert(key);
                       }
                    }
                }
            }
        },
        _ => { println!("error capture"); }
    }
    println!("=================");
    println!("End of simulation");
}


