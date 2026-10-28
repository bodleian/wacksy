use std::error::Error;
use std::fmt;
use std::str::FromStr;

#[derive(Copy, Clone, PartialEq, Debug)]
pub struct DateTime {
    year: u16,
    month: u8,
    day: u64,
    hour: u8,
    minute: u8,
    second: u64,
}
impl DateTime {
    #[inline]
    const fn is_leap_year(self) -> bool {
        return self.year.is_multiple_of(400)
            || (self.year.is_multiple_of(4) && !self.year.is_multiple_of(100));
    }
    pub fn to_compressed_string(self) -> String {
        return format!(
            "{}{:02}{:02}{:02}{:02}{:02}",
            self.year, self.month, self.day, self.hour, self.minute, self.second
        );
    }
}
impl fmt::Display for DateTime {
    // Display the datetime as an ISO8601/RFC3399 formatted string
    // The Z on the end here indicates it's UTC
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}-{:02}-{:02}T{:02}:{:02}:{:02}Z",
            self.year, self.month, self.day, self.hour, self.minute, self.second
        )
    }
}

impl FromStr for DateTime {
    type Err = DateTimeError;

    // Expects a string in ISO8601/RFC3399 format eg. YYYY-MM-DD HH:MM:SS
    fn from_str(datetime_string: &str) -> Result<Self, Self::Err> {

        // Parse each section of the string to respective datetime values.
        if let Some(Ok(year)) = datetime_string.get(0..4).map(str::parse::<u16>)
            && let Some(Ok(month)) = datetime_string.get(5..7).map(str::parse::<u8>)
            && let Some(Ok(day)) = datetime_string.get(8..10).map(str::parse::<u64>)
            && let Some(Ok(hour)) = datetime_string.get(11..13).map(str::parse::<u8>)
            && let Some(Ok(minute)) = datetime_string.get(14..16).map(str::parse::<u8>)
            && let Some(Ok(second)) = datetime_string.get(17..19).map(str::parse::<u64>)
        {
            return Ok(Self {
                year,
                month,
                day,
                hour,
                minute,
                second,
            });
        } else {
            Err(DateTimeError::ParsingError(datetime_string.to_owned()))
        }
    }
}

#[derive(Debug)]
pub enum DateTimeError {
    ParsingError(String),
}

impl fmt::Display for DateTimeError {
    fn fmt(&self, message: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ParsingError(datetime_string) => {
                // This message should indicate what format the datetime string should be in.
                // When propogating it up the chain, the caller should also give some
                // detail about where the error occurred.
                return write!(message, "Error: could not parse the {datetime_string}"); 
            }
        }
    }
}
impl Error for DateTimeError {
    // It would be a good idea at this point
    // to impl source() for DateTimeError
    // and use DateTimePart so we can idiomatically
    // use error.source() to refer to the specific
    // datetime section which couldn't be parsed.
}

impl Default for DateTime {
    // This default state for DateTime is the Unix epoch:
    // 1970-01-01T00:00:00Z
    fn default() -> Self {
        return Self {
            year: 1970,
            month: 1,
            day: 1,
            hour: 0,
            minute: 0,
            second: 0,
        };
    }
}
// This function takes seconds since unix epoch and returns
// iso-8601 formatted string
// Use this as a guide https://www.geeksforgeeks.org/dsa/convert-unix-timestamp-to-dd-mm-yyyy-hhmmss-format/
pub fn seconds_to_datetime(seconds_from_epoch: u64) -> DateTime {
    // Initialise datetime, starting with seconds_from_epoch
    let mut datetime = DateTime {
        second: seconds_from_epoch,
        ..Default::default()
    };

    // Divide total number of seconds by 86,400 (number of seconds in a day)
    // to get the number of days since the epoch
    datetime.day += datetime.second / 86_400;
    while datetime.day >= 365 {
        if datetime.is_leap_year() {
            if datetime.day < 366 {
                break;
            }
            datetime.day -= 366;
        } else {
            datetime.day -= 365;
        }
        // increment current year on every wind round the while loop
        // this could be done better though?
        datetime.year += 1;
    }

    let days_per_month: [u64; 12] = if datetime.is_leap_year() {
        // If the year is a leap year, the second month
        // February will have 29 days
        [31, 29, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31]
    } else {
        // If it's not a leap year, this is the normal
        // pattern of days per month
        [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31]
    };

    // Loop through days per month, increment months and subtract
    // days until only days are left. There's a better way of doing
    // this for sure!
    for days in &days_per_month {
        if datetime.day > *days {
            datetime.day -= days;
            datetime.month += 1;
        } else {
            break;
        }
    }

    // Converts the given number of Unix seconds to an hour, minute, and second.
    datetime.hour = (
        // Seconds remaining after dividing total seconds
        // by the number of seconds in a day (86,400)
        (datetime.second % 86_400)
            // Divide the remaining seconds by the number of
            // seconds in an hour (3,600)
            / 3_600
    ) as u8;

    datetime.minute = (
        // Seconds remaining after dividing total seconds
        // by 3,600 (the number of seconds in an hour)
        (datetime.second % 3_600)
            // Divide the remaining seconds by 60 (the number of
            // seconds in a minute)
            / 60
    ) as u8;

    // Seconds remaining after dividing the total number of
    // seconds by 60 (number of seconds in a minute)
    datetime.second %= 60;
    // Now that we've reassigned a value to datetime.seconds,
    // it no longer holds the total number of seconds
    // from unix epoch to now.

    return datetime;
}
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_leap_year() {
        for year in [
            1972, 1976, 1980, 1984, 1988, 1992, 1996, 2000, 2004, 2008, 2012, 2016, 2020, 2024,
        ] {
            let test_datetime = DateTime {
                year,
                ..Default::default()
            };
            assert!(test_datetime.is_leap_year());
        }
    }

    #[test]
    fn test_is_not_leap_year() {
        for year in [1970, 2001] {
            let test_datetime = DateTime {
                year,
                ..Default::default()
            };
            assert!(!test_datetime.is_leap_year());
        }
    }
    #[test]
    fn parse_datetime_from_string() {
        assert_eq!(
            DateTime::from_str("1970-01-01T00:00:00Z").unwrap(),
            DateTime::default()
        );
        assert_eq!(
            DateTime::from_str("2026-08-31T21:15:03Z").unwrap(),
            DateTime {
                year: 2026,
                month: 8,
                day: 31,
                hour: 21,
                minute: 15,
                second: 3,
            }
        );
        assert_eq!(
            DateTime::from_str("2026-09-01T08:04:26Z").unwrap(),
            DateTime {
                year: 2026,
                month: 9,
                day: 1,
                hour: 8,
                minute: 4,
                second: 26,
            }
        );
    }
    #[test]
    fn test_unix_epoch_to_datetime_string() {
        assert_eq!(seconds_to_datetime(0).to_string(), "1970-01-01T00:00:00Z");
        assert_eq!(
            seconds_to_datetime(1_788_210_903).to_string(),
            "2026-08-31T21:15:03Z"
        );
        assert_eq!(
            seconds_to_datetime(1_788_249_866).to_string(),
            "2026-09-01T08:04:26Z"
        );
    }
    #[test]
    fn test_unix_epoch_to_compressed_string() {
        assert_eq!(
            seconds_to_datetime(0).to_compressed_string(),
            "19700101000000"
        );
        assert_eq!(
            seconds_to_datetime(1_788_210_903).to_compressed_string(),
            "20260831211503"
        );
        assert_eq!(
            seconds_to_datetime(1_788_249_866).to_compressed_string(),
            "20260901080426"
        );
    }
    #[test]
    fn test_unix_epoch_to_datetime() {
        // Should be 1970-01-01T00:00:00Z
        assert_eq!(seconds_to_datetime(0), DateTime::default());
        // Should be 2026-08-31T21:15:03Z
        assert_eq!(
            seconds_to_datetime(1_788_210_903),
            DateTime {
                year: 2026,
                month: 8,
                day: 31,
                hour: 21,
                minute: 15,
                second: 3,
            }
        );
        // Should be 2026-09-01T08:04:26Z
        assert_eq!(
            seconds_to_datetime(1_788_249_866),
            DateTime {
                year: 2026,
                month: 9,
                day: 1,
                hour: 8,
                minute: 4,
                second: 26,
            }
        );
    }
}
