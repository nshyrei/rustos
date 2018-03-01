use core::cmp::Ordering;

pub fn log_align_down(base : usize, x : usize) -> usize {  
    match log_inner(base, x) {
        (result, Ordering::Greater) => result - 1,
        (result, _) => result
    }
}

pub fn log_align_up(base : usize, x : usize) -> usize {
    match log_inner(base, x) {
        (result, Ordering::Greater) => result + 1,
        (result, _) => result
    }
}

pub fn log2_align_down(x : usize) -> usize {
    log_align_down(2, x)
}

pub fn log2_align_up(x : usize) -> usize {
    log_align_up(2, x)
}

fn log_inner(base : usize, x : usize) -> (usize, Ordering) {  
    let mut result = 1;
    let mut accm = base;        

    loop {
        if accm == x {
            return (result, Ordering::Equal)
        }
        else if accm > x {
            return (result, Ordering::Greater)
        }
        else {
            accm *= base;
            result += 1;
        }
    }    
}

pub fn is_even(x : usize) -> bool {
    x % 2 == 0
}