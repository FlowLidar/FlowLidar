#include "sketch.h"
#include <bits/stdint-uintn.h>

using namespace std;

string path = "../data/test-8s.dat";
vector<trace_t> traces;
map<trace_t, uint32_t> ground_truth;
NZEsketch<uint32_t, uint32_t, (uint32_t) SLOT_NUM, (uint32_t) CM_DEPTH, (uint32_t) CM_WIDTH, (uint32_t) BLOOM_SIZE, (uint32_t) BLOOM_HASH_NUM> nze;
//NZEsketch<uint32_t, uint32_t, SLOT_NUM, CM_DEPTH, CM_WIDTH, BLOOM_SIZE, BLOOM_HASH_NUM> nze;

int readTraces(const char *path) {
	FILE *inputData = fopen(path, "rb");

	assert(inputData != NULL);

	traces.clear();
	char *strData = new char[KEY_T_SIZE];

	printf("Reading in data\n");

	while (fread(strData, KEY_T_SIZE, 1, inputData) == 1) {
		traces.push_back(trace_t(strData));
	}
	fclose(inputData);
	
	int size = traces.size();

	printf("Successfully read in %d packets\n", size);

	return size;
}

int main(int argc, char *argv[]) {
	//read the traces
	double slot_mem_factor, cms_mem_factor, bf_mem_factor;
	if (argc > 1) {
		path = argv[1];
	}

	int size = readTraces(path.c_str());
	//cout << "# packet (size) " << size << endl;
	//packet break number!
	int break_number = 100000000;
	double are = 0, perFlowARE = 0;
	double aae = 0, perFlowAAE = 0;
	int packetCnt = 0, staisfiedFlowCnt = 0;
	bool lastFlow = false;

	/********************* NZE sketch ***************************/

	//get the ground truth & insert
	for (int i = 0; i < size; ++i) {
		//if (i < 10)
		//	cout << traces[i].str << endl;
		if (ground_truth.find(traces[i]) != ground_truth.end()) {
			ground_truth[traces[i]] += 1;
		}
		else if (!lastFlow) {
			ground_truth[traces[i]] = 1;
			if (ground_truth.size() == break_number)
				lastFlow = true;
		}
		else {
			continue;
		}

		nze.insert((Key_t)traces[i].str);
		packetCnt++;
	}
	cout << "Insert " << packetCnt << " packets and " << ground_truth.size() << " flows" << endl;

	int zero_error = 0;
	for (auto it = ground_truth.begin(); it != ground_truth.end(); it++) {
		uint32_t ans = nze.query((Key_t)it->first.str);
		if (ans != it->second) {
			perFlowARE = fabs((double)ans - (double)it->second) / (double)it->second;
			perFlowAAE = fabs((double)ans - (double)it->second) ;
			are += perFlowARE;
			aae += perFlowAAE;
			if (perFlowARE <= 0.001) {
				staisfiedFlowCnt++;
			}
			// cout << ans << " " << it->second << endl;
		}
		else {
			staisfiedFlowCnt++;
			zero_error++;
		}
	}

	size_t nzeSize = nze.get_memory_usage();

	are /= ground_truth.size();
	aae /= ground_truth.size();
	cout << "ARE of NZE:" << are << endl;
	cout << "AAE of NZE:" << aae << endl;
	cout << "Satisfied flow proportion:" << (double)staisfiedFlowCnt / ground_truth.size() * 100 << "%" << endl;
	cout << "Zero erro flow proportion:" << (double)zero_error/ ground_truth.size() * 100 << "%" << endl;

	return 0;
}




