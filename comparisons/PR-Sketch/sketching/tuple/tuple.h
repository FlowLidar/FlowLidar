#ifndef __TUPLE_H__
#define __TUPLE_H__

#include <stdint.h>

#include "../utils/util.h"

typedef struct __attribute__((__packed__)) TupleKey{
	uint32_t src_ip;
	uint32_t dst_ip;
    	uint16_t src_port;
    	uint16_t dst_port;
    	// 1 bytes
    	uint8_t proto;

	inline bool operator < (const TupleKey &other) const;
	inline bool operator == (const TupleKey &other) const;
} tuple_key_t;

inline bool TupleKey::operator < (const TupleKey &other) const {
	return (src_ip < other.src_ip) || ((src_ip == other.src_ip) && (dst_ip < other.dst_ip)) || ((src_ip == other.src_ip && dst_ip == other.dst_ip) && (src_port < other.src_port)) || ((src_ip == other.src_ip && dst_ip == other.dst_ip && src_port == other.src_port) && (dst_port < other.dst_port)) || ((src_ip == other.src_ip && dst_ip == other.dst_ip && src_port == other.src_port && dst_port == other.dst_port) && (proto < other.proto));
}

inline bool TupleKey::operator == (const TupleKey &other) const {
	return (src_ip == other.src_ip) && (dst_ip == other.dst_ip) && (src_port == other.src_port) && (dst_port == other.dst_port) && (proto == other.proto);
}

typedef struct __attribute__ ((__packed__)) FlowKey {
	// 8 (4*2) bytes
    uint32_t src_ip;  // source IP address
    uint32_t dst_ip;
	// 4 (2*2) bytes
    uint16_t src_port;
    uint16_t dst_port;
    // 1 bytes
    uint8_t proto;
} flow_key_t;

#define TUPLE_NORMAL 0
#define TUPLE_PUNC   1
#define TUPLE_TERM   2
#define TUPLE_START  3

typedef struct __attribute__((__packed__)) Tuple {
    /**************************************
     * keys
     *************************************/
    flow_key_t key;

    /**************************************
     * values 
     *************************************/
    // 4 bytes
	int32_t size;			// inner IP datagram length (header + data)

    // 1 bytes
    // only used in multi-thread environment
    uint8_t flag;

	// 8 bytes
	double pkt_ts;				// timestamp of the packet
} tuple_t;

typedef struct __attribute__((__packed__)) RichTuple {
    flow_key_t key;

	int32_t size;
    int64_t index;
    double conf[104];
} rich_tuple_t;

#endif
