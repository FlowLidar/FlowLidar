/*
 * How to read a packet capture file.
 */

/*
 * Step 1 - Add includes
 */
#include <arpa/inet.h>
#include <cstdint>
#include <cstdio>
#include <cstdlib>
#include <cstring>
#include <netinet/in.h>
#include <set>
#include <map>
#include <tuple>
#include <netinet/ip.h>
#include <string>
#include <iostream>
#include <pcap.h>
#include <net/ethernet.h>
#include <utility>

using namespace std;

int diff_ms(timeval t1, timeval t2)
{
	return (((t1.tv_sec - t2.tv_sec) * 1000000) + (t1.tv_usec - t2.tv_usec))/1000;
}


struct rte_ipv4_hdr {
     union {
         uint8_t version_ihl;    
         struct {
             uint8_t version:4; 
             uint8_t ihl:4;     
         };
     };
     uint8_t  type_of_service;   
     uint16_t total_length;    
     uint16_t packet_id;       
     uint16_t fragment_offset; 
     uint8_t  time_to_live;      
     uint8_t  next_proto_id;     
     uint16_t hdr_checksum;    
     uint32_t src_addr;        
     uint32_t dst_addr;        
} __attribute__((__packed__)) ;

 struct rte_tcp_hdr {
     uint16_t src_port; 
     uint16_t dst_port; 
     uint32_t sent_seq; 
     uint32_t recv_ack; 
     uint8_t  data_off;   
     uint8_t  tcp_flags;  
     uint16_t rx_win;   
     uint16_t cksum;    
     uint16_t tcp_urp;  
 } __attribute__((__packed__)) ;
 
 struct rte_udp_hdr {
     uint16_t src_port;    
     uint16_t dst_port;    
     uint16_t dgram_len;   
     uint16_t dgram_cksum; 
 } __attribute__((__packed__)) ;

typedef std::tuple<uint32_t,uint32_t, uint16_t, uint16_t, uint8_t> flow_key_t;

int main(int argc, char *argv[])
{
	// ethernet header
	std::map<flow_key_t, uint32_t> flow_map;
	std::set<flow_key_t> flow_set;
	std::set<uint32_t> sip_set;
	struct ether_header ethHdr;
	struct rte_ipv4_hdr ipHdr;
	struct rte_tcp_hdr tcpHdr;
	struct rte_udp_hdr udpHdr;
	flow_key_t flow_key;
	uint32_t no_tcp_udp = 0;
	uint32_t tcp = 0;
	uint32_t udp = 0;

	if (argc != 3) {
		printf("Usage: ./pcap_analizer pcap_file output_file\n");
		exit(-1);
	}
	/*
	 * Step 3 - Create an char array to hold the error.
	 */

	char errbuff[PCAP_ERRBUF_SIZE];

	/*
	 * Step 4 - Open the file and store result in pointer to pcap_t
	 */

	// Use pcap_open_offline
	// http://www.winpcap.org/docs/docs_41b5/html/group__wpcapfunc.html#g91078168a13de8848df2b7b83d1f5b69
	pcap_t * pcap = pcap_open_offline(argv[1], errbuff);
	FILE* out = fopen(argv[2], "wb");

	/*
	 * Step 5 - Create a header and a data object
	 */

	// Create a header object:
	// http://www.winpcap.org/docs/docs_40_2/html/structpcap__pkthdr.html
	struct pcap_pkthdr *header;

	const u_char *data;

	/*
	 * Step 6 - Loop through packets and print them to screen
	 */
	u_int packetCount = 0;
	u_int tempCount = 0;
	char flow_key_binary[13];
	timeval first_ts;
	while (int returnValue = pcap_next_ex(pcap, &header, &data) >= 0)
	{
		// Show the packet number
		//if (packetCount % 100000 == 0)
		//	printf("Packet # %i\n", packetCount);
		if (packetCount==0) {
			//printf("first ts: %ld %ld\n",header->ts.tv_sec,header->ts.tv_usec); 
			first_ts=header->ts;
		}
		packetCount++;
		if (diff_ms(header->ts,first_ts)>1000) {
			//printf("last ts: %ld %ld\n",header->ts.tv_sec,header->ts.tv_usec);
		        break;
		}
		memcpy(&ipHdr, &data[0], sizeof(ipHdr));
		get<0>(flow_key) = ipHdr.src_addr;
		get<1>(flow_key) = ipHdr.dst_addr;
		memcpy(&flow_key_binary[0], &ipHdr.src_addr, 4);
		memcpy(&flow_key_binary[4], &ipHdr.dst_addr, 4);
		if (ipHdr.next_proto_id == 6) {
			memcpy(&tcpHdr, &data[sizeof(ipHdr)], sizeof(tcpHdr));
			get<2>(flow_key) = tcpHdr.src_port;
			get<3>(flow_key) = tcpHdr.dst_port;
			get<4>(flow_key) = 6;
			memcpy(&flow_key_binary[8], &tcpHdr.src_port, 2);
			memcpy(&flow_key_binary[10], &tcpHdr.dst_port, 2);
			flow_key_binary[12] = (uint8_t)6;
			flow_set.insert(flow_key);
			sip_set.insert(ipHdr.src_addr);
			fwrite(flow_key_binary, 13, 1, out);
			tcp++;

		} else if (ipHdr.next_proto_id == 17) {
			memcpy(&udpHdr, &data[sizeof(ipHdr)], sizeof(udpHdr));
			get<2>(flow_key) = udpHdr.src_port;
			get<3>(flow_key) = udpHdr.dst_port;
			get<4>(flow_key) = 17;
			udp++;
		} else {
			no_tcp_udp++;
		}

		tempCount++;
	}
	fclose(out);
	printf("#Flows (IPv4 TCP): %lu\n", flow_set.size());
	printf("#SIP Flows (IPv4 TCP SIP ): %lu\n", sip_set.size());
	cout << "no tcp udp " << no_tcp_udp << endl;
	cout << "tcp " << tcp << endl;
	cout << "udp " << udp << endl;
	cout << "packetCount " << packetCount << endl;
}
