#ifndef _PARAMETERS_H
#define _PARAMETERS_H

#ifndef KEY_T_SIZE
	#define KEY_T_SIZE 13
#endif
//parameters of CM sketch
#ifndef CM_DEPTH
	#define CM_DEPTH 4
#endif

#ifndef CM_WIDTH 
	#define CM_WIDTH 4000
#endif

//parameters of Bloom Filter
#ifndef BLOOM_SIZE
	#define BLOOM_SIZE 512000
#endif

#ifndef BLOOM_HASH_NUM
	#define BLOOM_HASH_NUM 4
#endif

//parameters of hash table
#ifndef SLOT_NUM
	#define SLOT_NUM (32000/21)
#endif

#ifndef EVICT_THRESHOLD 
	#define EVICT_THRESHOLD 1
#endif

//return value of hash table's insertion
#define HIT 0
#define MISS_EVICT 1
#define MISS_INSERT 2
#define NEW 3

#endif
