
/// Produce an EDT from binary image
pub fn edt(map: &[bool], shape: (usize, usize)) -> Vec<f64> {
    let horz_edt = horizontal_edt(map, shape);

    let vertical_scan = |x, y| {
        let total_edt = (0..shape.1).map(|y2| {
            let horz_val: f64 = horz_edt[x + y2 * shape.0];
            (y2 as f64 - y as f64).powf(2.) + horz_val.powf(2.)
        });
        total_edt.reduce(f64::min).unwrap()
    };

    let mut ret = vec![0.; shape.0 * shape.1];

    for x in 0..shape.0 {
        for y in 0..shape.1 {
            ret[x + y * shape.0] = vertical_scan(x, y);
        }
    }

    ret
}

fn horizontal_edt(map: &[bool], shape: (usize, usize)) -> Vec<f64> {
    let mut horz_edt = map
        .iter()
        .map(|b| ((*b as usize) * map.len()) as f64)
        .collect::<Vec<f64>>();

    let mut scan = |x, y, min_val: &mut f64, horz_edt: &mut Vec<f64>| {
        let f: f64 = horz_edt[x + y * shape.0];
        let next = *min_val + 1.;
        let v = f.min(next);
        horz_edt[x + y * shape.0] = v;
        *min_val = v;
    };

    for y in 0..shape.1 {
        let mut min_val = 0.;
        for x in 0..shape.0 {
            scan(x, y, &mut min_val, &mut horz_edt);
        }
        eprintln!("left {}: {:?}", y, &horz_edt[y * shape.0..(y + 1) * shape.0]);
        min_val = 0.;
        for x in (0..shape.0).rev() {
            scan(x, y, &mut min_val, &mut horz_edt);
        }
        eprintln!("rght {}: {:?}", y, &horz_edt[y * shape.0..(y + 1) * shape.0]);
    }

    horz_edt
}

#[cfg(test)]
mod test {
    use super::*;

    fn flatten<T>(nested: Vec<Vec<T>>) -> Vec<T> {
        nested.into_iter().flatten().collect()
    }

    fn test_map() -> Vec<bool> {
        let str_map = ["0000000000", "0001111000", "0011111110", "0001111000", "0000110000"];
        let map = flatten(
            str_map
                .iter()
                .map(|s| s.chars().map(|c| c == '1').collect::<Vec<_>>())
                .collect::<Vec<_>>(),
        );
        map
    }

    fn reshape(v: &Vec<f64>, shape: (usize, usize)) -> Vec<Vec<f64>> {
        let mut ret = vec![];

        for y in 0..shape.1 {
            ret.push(v[y * shape.0..(y + 1) * shape.0].into());
        }

        ret
    }

    fn print_2d(v: &[Vec<f64>]) {
        for row in v {
            for cell in row {
                if *cell == 16. {
                    print!("f");
                } else {
                    print!("{:.0}", cell);
                }
            }
            print!("\n");
        }
    }

    fn parse_edt_str(s: &[&str]) -> Vec<f64> {
        flatten(s
        .iter()
        .map(|s| s
            .chars()
            .map(|c| if c != 'f' { (c as u8 - '0' as u8) as f64 } else { 15. })
            .collect::<Vec<_>>())
        .collect::<Vec<_>>())
    }

    #[test]
    fn test_horizontal_edt() {
        let map = test_map();
        let str_edt = ["0000000000", "0001221000", "0012343210", "0001221000", "0000110000"];
        print_2d(&reshape(&horizontal_edt(&map, (map.len() / str_edt.len(), str_edt.len())), (str_edt[0].len(), str_edt.len())));
        assert_eq!(
            horizontal_edt(&map, (map.len() / str_edt.len(), str_edt.len())),
            parse_edt_str(&str_edt)
        );
    }

    #[test]
    fn test_edt() {
        let map = test_map();
        let str_edt = ["0001111000", "00149f9410", "0001441000", "0000110000"];
        let shape = (map.len() / str_edt.len(), str_edt.len());
        let edt = edt(&map, shape);
        eprintln!("edt({:?}): {:?}", shape, edt);
        assert_eq!(
            edt,
            parse_edt_str(&str_edt)
        );
    }
}
