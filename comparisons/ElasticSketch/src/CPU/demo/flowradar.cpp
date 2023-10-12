#include <stdio.h>
#include <string.h>
#include <stdlib.h>
#include <unordered_map>
#include <vector>

#include "../FlowRadar/insertable_iblt.h"
using namespace std;

#define START_FILE_NO 1
#define END_FILE_NO 1

uint32_t JSHash(uint8_t* key, unsigned int length) 
{
	unsigned int hash = 1315423911;
	unsigned int i = 0;
	for (i = 0; i < length; key++, i++)
		hash ^= ((hash << 5) + (*key) + (hash >> 2));
	return hash;
}

struct FIVE_TUPLE{	char key[13];	};
typedef vector<FIVE_TUPLE> TRACE;
TRACE traces[END_FILE_NO - START_FILE_NO + 1];

void ReadInTraces(const char *trace_prefix)
{
	for(int datafileCnt = START_FILE_NO; datafileCnt <= END_FILE_NO; ++datafileCnt)
	{
		char datafileName[100];
		sprintf(datafileName, "%s%d.dat", trace_prefix, datafileCnt - 1);
		printf("open file %s\n", datafileName);
		FILE *fin = fopen(datafileName, "rb");
		
		FIVE_TUPLE tmp_five_tuple;
		traces[datafileCnt - 1].clear();
		while(fread(&tmp_five_tuple, 1, 13, fin) == 13)
		{
			traces[datafileCnt - 1].push_back(tmp_five_tuple);
		}
		fclose(fin);

		printf("Successfully read in %s, %ld packets\n", datafileName, traces[datafileCnt - 1].size());
	}
	printf("\n");
}

int main(int argc, char* argv[])
{
	int TOT_MEM_IN_BYTES= (128 * 1024);
	//char filename[100]="../../../data/";
	char filename[100]="./c1-";
	if(argc==2) {
		strcpy(filename,argv[1]);
	}
	if(argc==3) {
		strcpy(filename,argv[1]);
		TOT_MEM_IN_BYTES=1024*atoi(argv[2]);
	}
	
	//ReadInTraces("../../../data/");
	ReadInTraces(filename);


	InsertableIBLT *fr = NULL;



	for(int datafileCnt = START_FILE_NO; datafileCnt <= END_FILE_NO; ++datafileCnt)
	{
		unordered_map<string, int> Real_Freq;
		unordered_map<uint32_t, int> Real_Freq2;
		fr = new InsertableIBLT(TOT_MEM_IN_BYTES);
		int packet_cnt = (int)traces[datafileCnt - 1].size();

		for(int i = 0; i < packet_cnt; ++i)
		{
			//fr->insert(*(uint32_t*)(traces[datafileCnt - 1][i].key));
			uint32_t flowID= JSHash((uint8_t*)(traces[datafileCnt - 1][i].key),13);
			
			bool exist=(fr->approximate_query(flowID)==-1);
			if ( (fr->num_flow <(8*fr->w_iblt)/10) || exist)
				fr->insert(flowID); 

			//string str((const char*)(traces[datafileCnt - 1][i].key), 4);
			uint32_t key;
			char cstring[100];
			memcpy(&key,traces[datafileCnt - 1][i].key, 4);
			sprintf(cstring,"%d",key);
			Real_Freq[string(cstring)]++;
			string s=string((const char*)(traces[datafileCnt - 1][i].key));
			s.resize(13);
			//Real_Freq2[s]++;
			Real_Freq2[flowID]++;
		}

		unordered_map<uint32_t, int> fr_result;
		fr->dump(fr_result);
		int decoded=0;
		double ARE = 0;
		double AAE = 0;
		int wrong = 0;
		int exact = 0;
		for( auto x : Real_Freq2)
		{
			if (fr_result.find(x.first)==fr_result.end()) continue;
			int est_val = fr_result[x.first];
			int dist = std::abs(x.second - est_val);
			double RE = dist * 1.0 / (x.second);
			ARE += RE; 
			AAE += abs(dist*1.0);
			if  (dist ==0) 
				exact++;
			decoded++;
		}
		//ARE /= (int)Real_Freq2.size();
		//AAE /= (int)Real_Freq2.size();
		ARE /= decoded;
		AAE /= decoded;

		printf("%d.dat: ARE=%.5lf\t", datafileCnt - 1, ARE);
		printf("%d.dat: AAE=%.5lf\t", datafileCnt - 1, AAE);
		printf("Number of flows: %lu\t", Real_Freq2.size());
		printf("Number of flows in the IBLT: %d\n", fr->num_flow);
		printf("Number of exact: %d (%f)\n", exact,exact/(0.0+Real_Freq2.size()));
		printf("Number of wrong: %d (%f)\n", wrong,wrong/(0.0+Real_Freq2.size()));
		printf("Number of decoded: %d (%f)\n", decoded,decoded/(0.0+Real_Freq2.size()));
		printf("used %d KB\n", TOT_MEM_IN_BYTES/1024);
	}
}	
