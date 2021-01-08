extern crate rand;

use linked_list::LinkedList;
use std::ops::Add;

#[derive(Debug)]
struct Bin {
    value: f64,
    count: usize,
}

/// A Histogram struct include a double linklist and some attributes for manage data.
/// using linkedList for fast insert and merge items in a sorted data structure
#[derive(Debug)]
pub struct Histogram {
    bins: LinkedList<Bin>,
    max_bins: usize,
    total: u64,
    min: Option<f64>,
    max: Option<f64>,
}

impl Default for Histogram {
    fn default() -> Self {
        Histogram::new(100)
    }
}
impl Histogram {
    /// Histogram is a data structure for calculation some statistic value from stream data,
    /// for example the mean、max、min、 q-quantiles and cumulative distribution etc.
    /// You must set the bucket number first, greater bucket number will bring more precision， but also
    /// bring more load for process. Default is 10, any number less than 10 will reset to default.
    /// # Examples
    ///
    /// ```
    ///  use crate::stream_histogram::Histogram;
    ///  let mut histogram = Histogram::new(20);
    ///  for i in 1..=100000 {
    ///       histogram.add(i as f64);
    ///  }
    /// ```
    pub fn new(max: usize) -> Histogram {
        let mut max_bin = max;
        if max_bin < 10 {
            max_bin = 10;
        };
        Histogram {
            bins: LinkedList::new(),
            max_bins: max_bin,
            total: 0,
            min: None,
            max: None,
        }
    }

    /// add method update histogram data structure from stream
    /// each time update a new value current value must be f64
    /// after update, the inner process will check the bucket number
    /// and merge two smallest gap bucket into one bucket.
    /// # Examples
    ///
    /// ```
    ///  use crate::stream_histogram::Histogram;
    ///  let mut histogram = Histogram::new(20);
    ///  for i in 1..=100000 {
    ///       histogram.add(i as f64);
    ///  }
    /// ```
    pub fn add(&mut self, number: f64) {
        self.total = self.total + 1;
        match self.min {
            Some(v) => {
                if number < v {
                    self.min = Some(number)
                }
            }
            _ => {
                self.min = Some(number);
            }
        }
        match self.max {
            Some(v) => {
                if number > v {
                    self.max = Some(number)
                }
            }
            _ => {
                self.max = Some(number);
            }
        }
        // println!(" {}{:?}{:?}", self.total, self.max, self.min);
        let mut cursor = self.bins.cursor();
        loop {
            match cursor.next() {
                Some(v) => {
                    if v.value == number {
                        v.count += 1;
                        return;
                    }
                    if v.value > number {
                        let new_bin = Bin {
                            value: number,
                            count: 1,
                        };
                        cursor.seek_backward(1);
                        cursor.insert(new_bin);
                        self.merge_bin();
                        return;
                    }
                }
                None => break,
            }
        }
        let new_bin = Bin {
            value: number,
            count: 1,
        };
        cursor.seek_backward(1);
        cursor.insert(new_bin);
        // do the merge job after finish the insert
        self.merge_bin();
    }

    /// q-quantiles are values that partition a finite set of values into q subsets of
    /// (nearly) equal sizes. for more detail please check
    /// [wikipedia](https://en.wikipedia.org/wiki/Quantile)
    #[allow(dead_code)]
    pub fn quantile(&self, q: f64) -> Option<f64> {
        let mut count = q * self.total as f64;
        for i in self.bins.iter() {
            count -= i.count as f64;
            // println!("{} {} {:?}", q, i.count, i.value);
            if count <= 0.0 {
                return Some(i.value);
            }
        }
        None
    }
    /// Cumulative distribution function(aka: cdf) returns the value of the cumulative
    /// distribution at value x. for more detail, please check [wikipedia](https://en.wikipedia.org/wiki/Cumulative_distribution_function)
    #[allow(dead_code)]
    pub fn cdf(&mut self, x: f64) -> Option<f64> {
        let mut count = 0;
        for i in self.bins.iter() {
            if i.value <= x {
                count += i.count
            }
        }
        if self.total == 0 {
            None
        } else {
            Some(count as f64 / self.total as f64)
        }
    }

    pub fn mean(&self) -> Option<f64> {
        if self.total == 0 {
            return None;
        }
        let mut sum = 0.0;
        for i in self.bins.iter() {
            sum += i.value * i.count as f64
        }
        Some(sum / self.total as f64)
    }
    #[allow(dead_code)]
    pub fn variance(&self) -> Option<f64> {
        if self.total == 0 {
            return None;
        }
        let mut sum = 0.0;
        let mean = self.mean()?;

        for i in self.bins.iter() {
            sum += i.count as f64 * (i.value - mean) * (i.value - mean);
        }
        Some(sum / self.total as f64)
    }

    fn merge_bin(&mut self) {
        if self.bins.len() <= self.max_bins {
            return;
        }
        let mut min_delta: f64 = 1e99;
        let mut min_delta_index = 0;
        let mut index = 0;
        let mut last_bin_value: f64 = 0.0;
        for i in self.bins.iter() {
            if index == 0 {
                last_bin_value = i.value;
                index += 1;
                continue;
            }

            let delta = i.value - last_bin_value;
            if delta < min_delta {
                min_delta = delta;
                min_delta_index = index;
            }
            index += 1;
            last_bin_value = i.value;
        }
        // must have a current and last bin
        let current_bin = self
            .bins
            .remove(min_delta_index)
            .expect("retrieve current bin fail");
        let last_bin = self
            .bins
            .remove(min_delta_index - 1)
            .expect("retrieve last bin fail");

        let total_count = current_bin.count + last_bin.count;
        let value = (current_bin.value * current_bin.count as f64
            + last_bin.value * last_bin.count as f64)
            / total_count as f64;
        let merged_bin = Bin {
            value,
            count: total_count,
        };
        self.bins.insert(min_delta_index - 1, merged_bin);
    }
    #[allow(dead_code)]
    /// to_string method print the result in a simple visulazation style,
    /// each line print the bucket and the size in a dot mode.
    /// # Examples
    ///
    /// ```
    ///  use rand::distributions::{Normal, Distribution};
    ///  use crate::stream_histogram::Histogram;
    ///  let normal = Normal::new(10.0, 10.0);
    ///  let mut histogram = Histogram::new(20);
    ///  for _i in 1..=100000 {
    ///       let v = normal.sample(&mut rand::thread_rng());
    ///       histogram.add(v as f64);
    ///  }
    ///  println!("{}", histogram.to_string());
    /// ```
    pub fn to_string(&self) -> String {
        let mut result = format!("Total: {}\n", self.total);
        for i in self.bins.iter() {
            let mut bar = format!("{}", i.value);
            let size = (i.count as f64 / self.total as f64 * 100.0) as usize;
            for _i in 1..size {
                bar += &*String::from(".");
            }
            result += &*(bar.add("\n"));
        }
        result
    }

    /// return a report from current histogram
    /// # Example
    /// ```
    /// use crate::stream_histogram::Histogram;
    /// let mut histogram = Histogram::new(10);
    /// for i in 1..=100 {
    ///     histogram.add(i as f64);
    /// }
    /// if let Some(report) = histogram.report(){
    ///    println!("{:?}", report);
    /// }
    /// ```
    pub fn report(&self) -> Option<HistogramReport> {
        HistogramReport::new(&self)
    }
}
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct HistogramReport {
    pub total: u64,
    pub mean: f64,
    pub max: f64,
    pub min: f64,
    pub percent99: f64,
    pub percent90: f64,
    pub percent50: f64,
}

#[allow(dead_code)]
impl HistogramReport {
    pub fn new(histogram: &Histogram) -> Option<Self> {
        Some(HistogramReport {
            total: histogram.total,
            mean: histogram.mean()?,
            max: histogram.max?,
            min: histogram.min?,
            percent99: histogram.quantile(0.99)?,
            percent90: histogram.quantile(0.90)?,
            percent50: histogram.quantile(0.50)?,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::Histogram;
    use crate::HistogramReport;

    #[test]
    fn test_histogram() {
        let mut histogram = Histogram::new(10);
        assert_eq!(histogram.max_bins, 10);
        histogram.add(1.0);
        assert_eq!(histogram.total, 1);
        assert_eq!(histogram.bins.len(), 1);

        // same value located in same bin
        histogram.add(1.0);
        assert_eq!(histogram.total, 2);
        assert_eq!(histogram.bins.len(), 1);

        histogram.add(2.0);
        assert_eq!(histogram.total, 3);
        assert_eq!(histogram.bins.len(), 2);

        let mut histogram = Histogram::new(10);
        for i in 1..=100 {
            histogram.add(i as f64);
        }
        assert_eq!(histogram.total, 100);
        assert_eq!(histogram.bins.len(), 10);
        assert_eq!(histogram.max_bins, 10);
    }

    #[test]
    fn test_statistics_function() {
        let mut histogram = Histogram::new(10);
        for i in 1..=100 {
            histogram.add(i as f64);
        }

        let mean = histogram.mean();
        assert_eq!(mean.is_none(), false);
        let mean = mean.unwrap();
        assert_eq!(mean, 50.5);

        let cdf = histogram.cdf(100.0);
        assert_eq!(cdf.is_none(), false);
        let cdf = cdf.unwrap();
        assert_eq!(cdf, 1.0);
        let cdf = histogram.cdf(50.0);
        assert_eq!(cdf.is_none(), false);
        let cdf = cdf.unwrap();
        assert_eq!(cdf, 0.45);

        let quantile = histogram.quantile(0.50);
        assert_eq!(quantile.is_none(), false);
        let quantile = quantile.unwrap();
        assert_eq!(quantile, 52.5);

        let variance = histogram.variance();
        assert_eq!(variance.is_none(), false);
        let variance = variance.unwrap();
        assert_eq!(variance, 823.765);
    }
    #[test]
    fn test_print() {
        let mut histogram = Histogram::new(10);
        for i in 1..=12 {
            histogram.add(i as f64);
        }
        println!("{}", histogram.to_string());
    }
    #[test]
    fn test_report() {
        let mut histogram = Histogram::new(10);
        for i in 1..=100 {
            histogram.add(i as f64);
        }
        let report = HistogramReport::new(&histogram);
        assert_eq!(report.is_none(), false);
        let report = report.unwrap();

        assert_eq!(report.max, 100.00);
        assert_eq!(report.min, 1.00);
        assert_eq!(report.mean, 50.5);
        assert_eq!(report.total, 100);
        assert_eq!(report.percent50, 52.5);
        assert_eq!(report.percent90, 86.0);
        assert_eq!(report.percent99, 96.5);
    }
}
