extern crate lstsq;
extern crate nalgebra as na;
use na::{DMatrix};
extern crate nalgebra_lapack as nala;
use nala::LU;
use std::ops::Mul;

fn back_substitution (u_matrix:na::DMatrix<f32>, y:na::DMatrix<f32>) -> na::DMatrix<f32> {

    //Get number of rows
    let n = u_matrix.nrows();
    //Allocating space for the solution vector
    let mut x= DMatrix::zeros(n,1);

    //Here we perform the back-substitution.  
    //Initializing with the last row.
    x[(n-1,0)] = y[(n-1,0)] / u_matrix[(n-1,n-1)]; 

    //Looping over rows in reverse (from the bottom  up),
    //starting with the second to last row, because  the
    //last row solve was completed in the last step.
    for i in (0..n-1).rev() {
        let mut sum =0.0;
        for j in i+1..n {
            sum += u_matrix[(i,j)]*x[(j,0)];
        }
        x[(i,0)] = (y[(i,0)] - sum ) / u_matrix[(i,i)];
    }
    x
}

fn forward_substitution(l_matrix:na::DMatrix<f32>, b:na::DMatrix<f32>) -> na::DMatrix<f32> {

    //Get number of columns
    let n = l_matrix.ncols();
    //Allocating space for the solution vector
    let mut y= DMatrix::zeros(n,1);

    //Here we perform the forward-substitution.
    //Initializing  with the first row.
    y[(0,0)] = b[(0,0)] / l_matrix[(0,0)]; 

    //Looping over rows in reverse (from the bottom  up),
    //starting with the second to last row, because  the
    //last row solve was completed in the last step.
    for i in 1..n {
        //y[i] = (b[i] - np.dot(L[i,:i], y[:i])) / L[i,i]
        let mut sum =0.0;
        for j in 0..i {
            sum += l_matrix[(i,j)]*y[(j,0)];
        }
        y[(i,0)] = (b[(i,0)] - sum ) / l_matrix[(i,i)];
    }
    y
}

pub fn approx_solve( a:na::DMatrix<f32>, b:na::DMatrix<f32>) -> na::DMatrix<f32> {

    //1. create augmented matrix composed by A|b|#1 in a row
    let mut aa_vec = Vec::new();
    let m_size=a.nrows();
    let n_size=a.ncols();
    for i in 0..m_size {
        aa_vec.push((a.row(i),b[(i,0)],a.row(i).sum()));
    }
    
    //2. sort augmented matrix
    //here I sort for b vector (CMS value). 
    //another option is sort for b/#1 in a row
    aa_vec.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());
    
    //3. expand the rows
    let mut i=0;
    let mut ii=0;
    let mut aa_matrix = DMatrix::zeros(0,n_size+2);
    let mut bb = DMatrix::zeros(0,1);
    for (r1,r2,r3) in aa_vec {
        //expand until TH --> added rows=i-ii
        if 100*(m_size+i-ii) < 105*n_size {
            for j in 0..n_size {
                if r1[j] ==1.0 {
                    aa_matrix=aa_matrix.insert_row(i,0.0);
                    bb=bb.insert_row(i,0.0);
                    aa_matrix[(i,j)] = 1.0;
                    aa_matrix[(i,n_size)] = r2/r3;
                    bb[(i,0)] = r2/r3;
                    aa_matrix[(i,n_size+1)] = 1.0;
                    i +=1;
                }
            }
        }
        else {
            aa_matrix=aa_matrix.insert_row(i,0.0);
            bb=bb.insert_row(i,0.0);
            for j in 0..n_size {
                aa_matrix[(i,j)] = r1[j];
            }
            aa_matrix[(i,n_size)] = r2;
            bb[(i,0)] = r2;
            aa_matrix[(i,n_size+1)] = r3;
            i +=1;
        }
        ii +=1;
    }
    //println!("eAA[{},{}] matrix is {:}",aa_matrix.nrows(),aa_matrix.ncols(),aa_matrix);


    aa_matrix=aa_matrix.remove_column(n_size+1);
    aa_matrix=aa_matrix.remove_column(n_size);

    // try to solve:
    let lapack_lu = LU::new(aa_matrix); //1.2 sec
    let b1=lapack_lu.p().transpose().mul(&bb);
    let y = forward_substitution(lapack_lu.l(),b1);
    let mut s =  back_substitution(lapack_lu.u(),y);
    
    for i in 0..s.nrows() {
        if s[(i,0)]<0.0 {
            s[(i,0)]=0.0; 
        }
    }
    s
}

pub fn approx_solve_qr( a:na::DMatrix<f32>, b:na::DMatrix<f32>) -> na::DMatrix<f32> {

    //1. create augmented matrix composed by A|b|#1 in a row
    let mut aa_vec = Vec::new();
    let m_size=a.nrows();
    let n_size=a.ncols();
    for i in 0..m_size {
        aa_vec.push((a.row(i),b[(i,0)],a.row(i).sum()));
    }
    
    //2. sort augmented matrix
    //here I sort for b vector (CMS value). 
    //another option is sort for b/#1 in a row
    aa_vec.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());
    
    //3. expand the rows
    let mut i=0;
    let mut ii=0;
    let mut aa_matrix = DMatrix::zeros(0,n_size+2);
    let mut bb = DMatrix::zeros(0,1);
    for (r1,r2,r3) in aa_vec {
        //expand until TH --> added rows=i-ii
        if 100*(m_size+i-ii) < 105*n_size {
            for j in 0..n_size {
                if r1[j] ==1.0 {
                    aa_matrix=aa_matrix.insert_row(i,0.0);
                    bb=bb.insert_row(i,0.0);
                    aa_matrix[(i,j)] = 1.0;
                    aa_matrix[(i,n_size)] = r2/r3;
                    bb[(i,0)] = r2/r3;
                    aa_matrix[(i,n_size+1)] = 1.0;
                    i +=1;
                }
            }
        }
        else {
            aa_matrix=aa_matrix.insert_row(i,0.0);
            bb=bb.insert_row(i,0.0);
            for j in 0..n_size {
                aa_matrix[(i,j)] = r1[j];
            }
            aa_matrix[(i,n_size)] = r2;
            bb[(i,0)] = r2;
            aa_matrix[(i,n_size+1)] = r3;
            i +=1;
        }
        ii +=1;
    }
    //println!("eAA[{},{}] matrix is {:}",aa_matrix.nrows(),aa_matrix.ncols(),aa_matrix);


    aa_matrix=aa_matrix.remove_column(n_size+1);
    aa_matrix=aa_matrix.remove_column(n_size);

    // try to solve:
    let decomp = aa_matrix.qr();
    let qt = decomp.q().transpose();
    let r = decomp.unpack_r();
    let mut s = DMatrix::zeros(n_size,1);

    match r.clone().try_inverse() {
        None => {},
        Some(rm1) => {
            let b1 = qt.mul(&bb);
            s=rm1.mul(&b1);
        },
    }
    for i in 0..s.nrows() {
        if s[(i,0)]<0.0 {
            s[(i,0)]=0.0; 
        }
    }
    s
}

