// Instead of returning Option, this should have proper error handling.
pub fn create_surt(url: &str) -> Option<String> {
    let url_without_protocol = match url {
        url if url.starts_with("https") => url.get(8..),
        url if url.starts_with("http") => url.get(7..),
        // URLs starting with urn are not surt-able.
        url if url.starts_with("urn") => return None,
        _ => None,
    }?;
    let url_split = url_without_protocol.split_once('/')?;
    let mut host: Vec<&str> = url_split.0.split('.').collect();
    host.reverse();
    let host_reversed = host.join(",");
    return Some(format!("{host_reversed})/{}", url_split.1));
}

#[test]
fn valid_surt() {
    let test_cases = [
        ("http://www.archive.org/", "org,archive,www)/"),
        (
            "https://thehtml.review/04/ascii-bedroom-archive/",
            "review,thehtml)/04/ascii-bedroom-archive/",
        ),
        ("http://archive.org/", "org,archive)/"),
        ("http://archive.org/goo/", "org,archive)/goo/"),
        ("http://archive.org/goo/?", "org,archive)/goo/?"),
        ("http://archive.org/goo", "org,archive)/goo"),
    ];

    for test_case in test_cases {
        let surt_parsed_url = create_surt(test_case.0).unwrap();
        assert_eq!(surt_parsed_url, test_case.1);
    }

    let invalid_test_cases = ["www.archive.org", "archive.org", "urn:pageinfo:archive.org"];
    for test_case in invalid_test_cases {
        let surt_parsed_url = create_surt(test_case);
        assert_eq!(surt_parsed_url, None);
    }
}
