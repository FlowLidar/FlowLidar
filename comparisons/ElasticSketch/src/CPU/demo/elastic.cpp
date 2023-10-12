#include <stdio.h>
#include <string.h>
#include <stdlib.h>
#include <unordered_map>
#include <vector>

#include "../elastic/ElasticSketch.h"
using namespace std;

#define START_FILE_NO 1
#define END_FILE_NO 1


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
	char filename[100]="../../../data/";
	bool sparse_flag;
	if(argc==2) {
		int flag=atoi(argv[1]);
		sparse_flag= (flag==1) ? true : false;
	}
	if(argc==3) {
		int flag=atoi(argv[1]);
		sparse_flag= (flag==1) ? true : false;
		strcpy(filename,argv[2]);
	}
	
	//ReadInTraces("../../../data/");
	ReadInTraces(filename);

#define SCALING 1
#define KEYSIZE 128 //96 160 288 416
//#define KEYSIZE 416
//#define BUCKET_NUM 2400*SCALING 
//#define BUCKET_NUM 400*SCALING 
//#define KEYSIZE 288
//#define BUCKET_NUM 500*SCALING 
#define HEAVY_MEM (32 * 1024* SCALING)
//#define LIGHT_MEM (450 * 1024* SCALING)
#define BUCKET_NUM (HEAVY_MEM / (8*16)) // 8 buckets of 16 bytes 
//#define LIGHT_MEM (128 * 1024)*SCALING -BUCKET_NUM*KEYSIZE
#define LIGHT_MEM (96 * 1024* SCALING)
#define TOT_MEM_IN_BYTES (128 * 1024 * SCALING )

	ElasticSketch<BUCKET_NUM, LIGHT_MEM> *elastic = NULL;
    

	for(int datafileCnt = START_FILE_NO; datafileCnt <= END_FILE_NO; ++datafileCnt)
	{
		unordered_map<string, int> Real_Freq;
		unordered_map<string, int> Real_Freq2;
		elastic = new ElasticSketch<BUCKET_NUM, LIGHT_MEM>(sparse_flag);
		int packet_cnt = (int)traces[datafileCnt - 1].size();
		
		for(int i = 0; i < packet_cnt; ++i)
		{
			elastic->insert((uint8_t*)(traces[datafileCnt - 1][i].key));
			// elastic->quick_insert((uint8_t*)(traces[datafileCnt - 1][i].key));

			//string str((const char*)(traces[datafileCnt - 1][i].key), 4);
			uint32_t key;
			char cstring[100];
			memcpy(&key,traces[datafileCnt - 1][i].key, 4);
			sprintf(cstring,"%d",key);
			Real_Freq[string(cstring)]++;
			string s=string((const char*)(traces[datafileCnt - 1][i].key));
			s.resize(13);
			Real_Freq2[s]++;
		}
		double ARE = 0;
		double AAE = 0;
		double maxRE=0; 
		int maxAE=0;
		int wrong = 0;
		int near_exact = 0;
		int exact = 0;
		for(unordered_map<string, int>::iterator it = Real_Freq2.begin(); it != Real_Freq2.end(); ++it)
		{
			int est_val = elastic->query((uint8_t*)(it->first).c_str());
			int dist = std::abs(it->second - est_val);
			double RE = dist * 1.0 / (it->second);
			ARE += RE; 
			AAE += abs(dist*1.0);
			if  (dist ==0) 
				exact++;
		}
		ARE /= (int)Real_Freq2.size();
		AAE /= (int)Real_Freq2.size();

		printf("%d.dat: ARE=%.5lf\t", datafileCnt - 1, ARE);
		printf("%d.dat: AAE=%.5lf\t", datafileCnt - 1, AAE);
		printf("Number of flows: %lu\t", Real_Freq2.size());
		printf("Number of exact: %d (%f)\n", exact,exact/(1.0+Real_Freq2.size()));
		printf("stat: %f %f %f %lu\n",100*exact/(1.0+Real_Freq2.size()),AAE,ARE,Real_Freq2.size());
	}
}	
