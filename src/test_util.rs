pub(crate) fn test_map() -> Vec<bool> {
    let str_map = [
        "0000000000",
        "0001111000",
        "0011111110",
        "0011111100",
        "0001111000",
    ];
    let map = flatten(
        str_map
            .iter()
            .map(|s| s.chars().map(|c| c == '1').collect::<Vec<_>>())
            .collect::<Vec<_>>(),
    );
    map
}

pub(crate) fn reshape(v: &Vec<f64>, shape: (usize, usize)) -> Vec<Vec<f64>> {
    let mut ret = vec![];

    for y in 0..shape.1 {
        ret.push(v[y * shape.0..(y + 1) * shape.0].into());
    }

    ret
}

pub(crate) fn print_2d(v: &[Vec<f64>]) {
    for row in v {
        for cell in row {
            if *cell == 16. {
                print!("f");
            } else {
                print!("{:.1}", cell);
            }
        }
        print!("\n");
    }
}

pub(crate) fn flatten<T>(nested: Vec<Vec<T>>) -> Vec<T> {
    nested.into_iter().flatten().collect()
}

pub(crate) fn parse_edt_str(s: &[&str]) -> Vec<f64> {
    flatten(
        s.iter()
            .map(|s| {
                s.chars()
                    .map(|c| {
                        if c != 'f' {
                            (c as u8 - '0' as u8) as f64
                        } else {
                            15.
                        }
                    })
                    .collect::<Vec<_>>()
            })
            .collect::<Vec<_>>(),
    )
}
