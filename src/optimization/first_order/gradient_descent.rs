use std::default::Default;
use std::fmt::Debug;

use crate::math::num::FloatExt;
use crate::linalg::Matrix;
use crate::optimization::{F, DF};
use crate::optimization::line_search::LineSearchMethod;
use crate::optimization::first_order::{FirstOrderOptimizer, OptimizerResult};

pub struct GradientDescent<T: FloatExt> {
    pub max_iter: usize,
    pub g_rtol: T,
    pub g_atol: T
}

impl<T: FloatExt> Default for GradientDescent<T> {
    fn default() -> Self {
        GradientDescent {
            max_iter: 10000,
            g_rtol: T::epsilon().sqrt(),
            g_atol: T::epsilon()
        }
     }
}

impl<T: FloatExt + Debug> FirstOrderOptimizer<T> for GradientDescent<T>
{

    fn optimize<'a, X: Matrix<T>, LS: LineSearchMethod<T>>(&self, f: &'a F<T, X>, df: &'a DF<X>, x0: &X, ls: &'a LS) -> OptimizerResult<T, X> {        

        let mut x = x0.clone();     
        let mut fx = f(&x);

        let mut gvec = x0.clone();   
        let mut gnorm = gvec.norm2();        

        let gtol = (gvec.norm2() * self.g_rtol).max(self.g_atol);        

        let mut iter = 0;
        let mut alpha = T::one();        
        df(&mut gvec, &x);         

        while iter < self.max_iter && (iter == 0 || gnorm > gtol) {
            iter += 1;
                        
            let mut step = gvec.negative();

            let f_alpha = |alpha: T| -> T {
                let mut dx = step.clone();
                dx.mul_scalar_mut(alpha);
                f(&dx.add_mut(&x)) // f(x) = f(x .+ gvec .* alpha)
            };

            let df_alpha = |alpha: T| -> T {                
                let mut dx = step.clone();
                let mut dg = gvec.clone();
                dx.mul_scalar_mut(alpha);
                df(&mut dg, &dx.add_mut(&x)); //df(x) = df(x .+ gvec .* alpha)
                gvec.vector_dot(&dg)
            };

            let df0 = step.vector_dot(&gvec);            

            let ls_r = ls.search(&f_alpha, &df_alpha, alpha, fx, df0);
            alpha = ls_r.alpha;
            fx = ls_r.f_x;
            x.add_mut(&step.mul_scalar_mut(alpha));         
            df(&mut gvec, &x);            
            gnorm = gvec.norm2();            
        }  

        let f_x = f(&x);      

        OptimizerResult{
            x: x,
            f_x: f_x,
            iterations: iter
        }
    }
}

#[cfg(test)]
mod tests {    
    use super::*; 
    use crate::linalg::naive::dense_matrix::*;
    use crate::optimization::line_search::Backtracking;
    use crate::optimization::FunctionOrder;

    #[test]
    fn gradient_descent() { 

        let x0 = DenseMatrix::vector_from_array(&[-1., 1.]);
        let f = |x: &DenseMatrix<f64>| {                
            (1.0 - x.get(0, 0)).powf(2.) + 100.0 * (x.get(0, 1) - x.get(0, 0).powf(2.)).powf(2.)
        };

        let df = |g: &mut DenseMatrix<f64>, x: &DenseMatrix<f64>| {                                         
            g.set(0, 0, -2. * (1. - x.get(0, 0)) - 400. * (x.get(0, 1) - x.get(0, 0).powf(2.)) * x.get(0, 0));
            g.set(0, 1, 200. * (x.get(0, 1) - x.get(0, 0).powf(2.)));                
        };

        let mut ls: Backtracking<f64> = Default::default();
        ls.order = FunctionOrder::THIRD;
        let optimizer: GradientDescent<f64> = Default::default();
        
        let result = optimizer.optimize(&f, &df, &x0, &ls);

        println!("{:?}", result);
        
        assert!((result.f_x - 0.0).abs() < 1e-5);
        assert!((result.x.get(0, 0) - 1.0).abs() < 1e-2);
        assert!((result.x.get(0, 1) - 1.0).abs() < 1e-2);

    }

}