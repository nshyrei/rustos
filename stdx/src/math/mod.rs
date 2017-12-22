pub fn log(base : usize, x : usize) -> usize {  
        let mut result = 0;
        let mut accm = base;        

        while accm <= x {
            accm *= base;
            result += 1;
        }

        result
    }

pub fn log2(x : usize) -> usize {
    log(2, x)
}