use std::hash::Hash;
use std::hash::Hasher;
use std::cmp::min;
#[derive(Clone)]
pub struct CMS {
    filter_size: usize,
    hash_function_count: usize,
    pub filter_bins: Vec<u32>,
    num_blocks:usize,
}


impl CMS {
    pub fn build_cms(filter_size: usize, hash_function_count: usize, num_blocks:usize) -> CMS {
        let filter_bins = vec![0; filter_size*hash_function_count*num_blocks];
        CMS {
            filter_size, 
            hash_function_count,
            filter_bins,
            num_blocks,
        }
    }

    pub fn index<T: Hash>(&self, key: T) -> (usize,[i32;4]) {
        let mut result = (0,[-1;4]);
        let range = 0..self.hash_function_count;
        let hash_function_range = self.filter_size; 
        
        let mut hasher = std::collections::hash_map::DefaultHasher::default();
        "block".hash(&mut hasher);
        key.hash(&mut hasher);
        let block:usize = (hasher.finish() as usize) % self.num_blocks;
        result.0 = block;

        for i in range {
            let mut hasher = std::collections::hash_map::DefaultHasher::default();
            i.hash(&mut hasher);
            key.hash(&mut hasher);
            let mut index = hasher.finish() as usize;
            index = index % hash_function_range;
            index = index + i*hash_function_range; 
            result.1[i] = index as i32;
        }
        result
    }
    
    pub fn insert<T: Hash>(&mut self, key: T) {
        let range = 0..self.hash_function_count;
        let hash_function_range = self.filter_size; 
        let block_size = self.filter_size*self.hash_function_count; 

        let mut hasher = std::collections::hash_map::DefaultHasher::default();
        "block".hash(&mut hasher);
        key.hash(&mut hasher);
        let block:usize = (hasher.finish() as usize) % self.num_blocks;

        for i in range {
            let mut hasher = std::collections::hash_map::DefaultHasher::default();
            i.hash(&mut hasher);
            key.hash(&mut hasher);
            let mut index = hasher.finish() as usize;
            index = index % hash_function_range;
            index = index + i*hash_function_range; 
            self.filter_bins[block*block_size+index] = self.filter_bins[block*block_size+index] + 1 ;
        }
    }

    pub fn query<T: Hash>(&self, key: T) -> u32 {
        let mut result= u32::MAX;
        let range = 0..self.hash_function_count;
        let hash_function_range = self.filter_size; 
        let block_size:usize = self.filter_size*self.hash_function_count; 

        let mut hasher = std::collections::hash_map::DefaultHasher::default();
        "block".hash(&mut hasher);
        key.hash(&mut hasher);
        let block:usize = (hasher.finish() as usize) % self.num_blocks;

        for i in range {
            let mut hasher = std::collections::hash_map::DefaultHasher::default();
            i.hash(&mut hasher);
            key.hash(&mut hasher);
            let mut index = hasher.finish() as usize;
            index = index % hash_function_range;
            index = index + i*hash_function_range; 
            result = min(result,self.filter_bins[block*block_size+index]);
            }
        return result;
    }

    pub fn clear(&mut self) {
        for x in self.filter_bins.iter_mut() {
            *x = 0;
        }
    }
    pub fn get_load(&self) -> f32 {
        self.filter_bins.iter().map(|x| if *x>0 {1.0} else {0.0} ).sum::<f32>() / (self.filter_size *self.hash_function_count * self.num_blocks) as f32
    }

    pub fn get_block_load(&self, block:usize) -> f32 {
        let block_size:usize = self.filter_size*self.hash_function_count; 
        let mut sum=0.0;
        for i in 0..block_size {
            if self.filter_bins[block*block_size+i]>0 {
                sum += 1.0;
            }
        }
        sum /(self.filter_size *self.hash_function_count) as f32   
    }


    pub fn dump(&mut self) {
        for (n,x) in self.filter_bins.iter().enumerate() {
            println!("row [{}]={}",n, x);
        }
    }
}
