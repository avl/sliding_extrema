/*!
This is a simple implementation of the algorithm described by 'adamax' at:

https://stackoverflow.com/questions/4802038/implement-a-queue-in-which-push-rear-pop-front-and-get-min-are-all-consta

It implements a Queue with push and pop, with FIFO semantics, and a get_extrema() method, all with amortized O(1) time complexity.

The get_extrema method returns the extrema of all items in the queue, using a user-supplied metric. Examples
of extrema-functions are max, min, but not 'average' or 'mean'.

This structure can be used to implement a super-efficient max/min function for a sliding window of many samples.

Example:

```
extern crate sliding_extrema;
use sliding_extrema::SlidingExtrema;

let mut queue = SlidingExtrema::new(|a:&u32,b:&u32|(*a).max(*b));

queue.push(42);
queue.push(15);
queue.push(8);

assert_eq!(queue.get_extrema().unwrap(),42);

queue.pop();

assert_eq!(queue.get_extrema().unwrap(),15);


```

The structure is covered by an automatic fuzz-test, that should provide 100% test coverage.


*/

struct Minstack<T> {
    data : Vec<(T,T)>,    
}

impl<T:Clone> Minstack<T> {
    fn new() -> Minstack<T> {
        Minstack::<T> {
            data : Vec::new()
        }
    }
    fn len(&self) -> usize {
        self.data.len()
    }
    fn push<F:Fn(&T,&T) -> T>(&mut self, value : T, extrema_fun : &F) {
        if self.data.len()==0 {
            let temp = value.clone();
            self.data.push((value,temp));
        } else {
            let new_extrema = extrema_fun(&self.data.last().unwrap().1,&value);
            self.data.push((value,new_extrema));                        
        }                
    }
    fn pop(&mut self) -> Option<T> {
        self.data.pop().map(|x|x.0)
    }
    fn get_extrema(&self) -> Option<&T> {
        self.data.last().map(|x|&x.1)
    }

}

pub struct SlidingExtrema<T,F> {
    push_stack : Minstack<T>,
    pop_stack : Minstack<T>,
    extrema_fun : F,
}


impl<T:Clone,F:Fn(&T,&T) -> T> SlidingExtrema<T,F> {

    pub fn new(f:F) -> SlidingExtrema<T,F> {
        SlidingExtrema {
            push_stack : Minstack::new(),
            pop_stack : Minstack::new(),
            extrema_fun : f
        }
    }
    
    pub fn len(&self) -> usize {
        self.push_stack.len() + self.pop_stack.len()
    }
    
    pub fn get_extrema(&self) -> Option<T> {
        if self.push_stack.len() == 0 && self.pop_stack.len() == 0 {
            return None;                    
        }
        if self.push_stack.len() > 0 && self.pop_stack.len() == 0 {
            return Some(self.push_stack.get_extrema().unwrap().clone());
        }
        if self.push_stack.len() == 0 && self.pop_stack.len() > 0 {
            return Some(self.pop_stack.get_extrema().unwrap().clone());
        }
        Some((self.extrema_fun)(self.push_stack.get_extrema().unwrap(),self.pop_stack.get_extrema().unwrap()))
    }

    pub fn push(&mut self, value : T) {
        self.push_stack.push(value,&self.extrema_fun);
    }    
    pub fn pop(&mut self) -> Option<T> {
        if self.pop_stack.len()==0 {
            while self.push_stack.len() > 0 {
                let temp = self.push_stack.pop().unwrap();
                self.pop_stack.push(temp,&self.extrema_fun);
            }                        
        }
        self.pop_stack.pop()      
    }    
    
    
}





#[cfg(test)]
mod tests {
    extern crate rand;
    use ::SlidingExtrema;
    use self::rand::{thread_rng, Rng};

    fn test_iter() {
        let num_initial_items = thread_rng().gen_range(0,20);
        let num_random_ops = thread_rng().gen_range(0,60);
        let mut a = SlidingExtrema::new(|a:&u32,b:&u32|(*a).min(*b));
        let mut b = Vec::new();
        for _ in 0..num_initial_items {
            let value = thread_rng().gen_range(0,10);
            a.push(value);
            b.push(value);    
            assert_eq!(a.get_extrema().unwrap(), b.iter().fold(10000,|a,b|a.min(*b)));    
            
        }
        
        for _ in 0..num_random_ops {
            assert_eq!(a.len(),b.len());
            if thread_rng().gen_range(0,2) == 0 {
                //insert
                let value = thread_rng().gen_range(0,10);
                a.push(value);
                b.push(value);        
                assert_eq!(a.get_extrema().unwrap(), b.iter().fold(10000,|a,b|a.min(*b)));    
            } else {
                if b.len() > 0 {
                    assert_eq!(a.get_extrema().unwrap(), b.iter().fold(10000,|a,b|a.min(*b)));    
                    let bpop = b.remove(0);
                    assert_eq!(a.pop().unwrap(),bpop);
                    
                } else {
                    assert_eq!(None,a.pop());
                    assert_eq!(None,a.get_extrema());
                }
            }
        }        
    }
    #[test]
    fn fuzz() {
        for _ in 0..1000 {
            test_iter();
        }
        
    }
    #[test]
    fn min_and_max() {
        let mut t = SlidingExtrema::new(|a:&(u32,u32),b:&(u32,u32)|((a.0).min(b.0),(a.1.max(b.1))));
        t.push((1,1));
        assert_eq!(t.get_extrema().unwrap(),(1,1));
        t.push((3,3));
        assert_eq!(t.get_extrema().unwrap(),(1,3));
        t.push((2,2));                
        assert_eq!(t.get_extrema().unwrap(),(1,3));
        t.pop();
        assert_eq!(t.get_extrema().unwrap(),(2,3));                
    }
    
    #[test]
    fn it_works() {
        let mut t = SlidingExtrema::new(|a:&u32,b:&u32|(*a).min(*b));
        assert_eq!(None,t.get_extrema());
        t.push(42);         
        assert_eq!(Some(42),t.get_extrema());
        t.push(15);         
        assert_eq!(Some(15),t.get_extrema());
        t.push(17);
        assert_eq!(Some(15),t.get_extrema());
        assert_eq!(42,t.pop().unwrap());
        assert_eq!(Some(15),t.get_extrema());
        assert_eq!(15,t.pop().unwrap());
        assert_eq!(Some(17),t.get_extrema());
        assert_eq!(17,t.pop().unwrap());
        assert_eq!(None,t.get_extrema());
        
        assert_eq!(None,t.pop());
    }
}
