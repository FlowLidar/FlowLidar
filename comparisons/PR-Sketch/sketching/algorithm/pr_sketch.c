#include "./pr_sketch.h"

#include "../utils/util.h"
#include "../hash/hash.h"
#include "../utils/equation_solver.h"
#include <math.h>

BFCBF::BFCBF(uint32_t bf_space, uint32_t measure_space, uint32_t bf_hash_num, uint32_t cbf_hash_num) {
	this->bf_hash_num = bf_hash_num; // hash num of bloom filter
	this->cbf_hash_num = cbf_hash_num; // hash num of light part

	bf_bucketnum = bf_space * 32; // each int contains 32 bits
	bf_vector.resize(bf_bucketnum, false);

	//cbf_bucketnum = measure_space; // 1 int is the size of light part counter
	cbf_bucketnum = measure_space * 2; // 16 bits counters -> double the # words
	cbf_vector.resize(cbf_bucketnum, 0);

	bf_kick_tuplekey_vector.reserve(200000); // Set capacity to reduce the time of dynamic allocating
}

void BFCBF::update(tuple_key_t cur_key) {
	// Update bloom filter
	bool status = true;
	for (uint32_t bf_row_idx = 0; bf_row_idx < bf_hash_num; bf_row_idx++) {
		uint32_t bf_hash_idx = myhash(cur_key, bf_row_idx, bf_bucketnum).hash_idx;
		if (!bf_vector[bf_hash_idx]) {
			status = false;
			bf_vector[bf_hash_idx] = true;
		}
	}

	// Kick small flow to controller
	if (!status) {
		//tuple_key_t tmp_tuplekey;
		//tmp_tuplekey.src_ip = cur_key.src_ip;
		//tmp_tuplekey.dst_ip = cur_key.dst_ip;

		bf_kick_tuplekey_vector.push_back(cur_key);
	}

	// Update CBF part counter
	for (uint32_t row_idx = 0; row_idx < cbf_hash_num; row_idx++) {
		hash_result_t hash_result = myhash(cur_key, row_idx, cbf_bucketnum);
		cbf_vector[hash_result.hash_idx] += 1; 
		// 16 bits counters 
		//cbf_vector[hash_result.hash_idx] = cbf_vector[hash_result.hash_idx] > (pow(2, 16) - 1) ? (pow(2, 16) - 1) : cbf_vector[hash_result.hash_idx] ;
	}
}

uint32_t BFCBF::estimate(tuple_key_t cur_key) {
	// Equation Solver
	if (!is_solve) {
		equation_solve();
	}
	if (solution.find(cur_key) != solution.end()) {
		return solution[cur_key];
	}
	else {
		// return 0 if key not found -> untracked flow
		return 0;
	}
	
	// Traditional Estimator
	//uint32_t result = traditional_estimate(cur_key);
	//return result;
}

set<tuple_key_t> BFCBF::get_tuplekeys() {
	set<tuple_key_t> result;
	for (uint32_t i = 0; i < bf_kick_tuplekey_vector.size(); i++) {
		result.insert(bf_kick_tuplekey_vector[i]);
	}
	return result;
}

uint32_t BFCBF::get_transmission_pkts() {
	uint32_t nbyte = bf_vector.size()/8 + cbf_vector.size()*2;
	nbyte += bf_kick_tuplekey_vector.size()*8;
	uint32_t npkt = (nbyte - 1) / 1500 + 1;
	return npkt;
}

uint32_t BFCBF::get_transmission_bytes() {
	uint32_t nbyte = bf_vector.size()/8 + cbf_vector.size()*2;
	nbyte += bf_kick_tuplekey_vector.size()*8;
	return nbyte;
}

uint32_t BFCBF::traditional_estimate(tuple_key_t cur_key) {
	uint32_t result = 0;
	for (uint32_t row_idx = 0; row_idx < cbf_hash_num; row_idx++) {
		hash_result_t hash_result = myhash(cur_key, row_idx, cbf_bucketnum);
		uint32_t tmp = cbf_vector[hash_result.hash_idx];
		if (row_idx == 0 || tmp < result) {
			result = tmp;
		}
	}
	return result;
}

void BFCBF::equation_solve() {
	set<tuple_key_t> tuplekeys = get_tuplekeys();
	vector<tuple_key_t> tuplekey_vector;
	for (set<tuple_key_t>::iterator iter = tuplekeys.begin(); iter != tuplekeys.end(); iter++) {
		tuplekey_vector.push_back(*iter);
	}

	uint32_t nrow = cbf_vector.size();
	uint32_t ncol = tuplekey_vector.size();
	vector<uint32_t> rows;
	vector<uint32_t> cols;
	vector<uint32_t> values;
	for (uint32_t i = 0; i < ncol; i++) {
		tuple_key_t tmp_tuplekey = tuplekey_vector[i];
		for (uint32_t j = 0; j < cbf_hash_num; j++) {
			uint32_t hashidx = myhash(tmp_tuplekey, j, cbf_bucketnum).hash_idx;
			rows.push_back(hashidx);
			cols.push_back(i);
			values.push_back(1);
		}
	}

	// For DEBUG
	//equation_dump(rows, cols, values, nrow, ncol, cbf_vector);
	//exit(0);

	vector<uint32_t> result = solve(rows, cols, values, nrow, ncol, cbf_vector);
	//exit(0); // Memory test: 60 MB for 100K equation solver
	//vector<uint32_t> result = load_matlab_solution(); // For Debug
	for (uint32_t i = 0; i < result.size(); i++) {
		solution.insert(std::pair<tuple_key_t, uint32_t>(tuplekey_vector[i], result[i]));
	}
	is_solve = true;
}
