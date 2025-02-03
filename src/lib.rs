use serde_derive::Deserialize;
use serde_derive::Serialize;
use time::OffsetDateTime;

#[derive(Debug, Deserialize)]
struct Dat {
    data: Data,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Data {
    pub city: String,
    pub state: String,
    pub country: String,
    pub location: Location,
    pub current: Current,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Location {
    /// Lat, Lon
    pub coordinates: (f64, f64),
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Current {
    pub pollution: Pollution,
    pub weather: Weather,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Pollution {
    #[serde(rename = "ts")]
    #[serde(with = "time::serde::iso8601")]
    pub timestamp: OffsetDateTime,
    // AQI value based on US EPA standard
    #[serde(rename = "aqius")]
    pub aqi_us: u32,
    /// main pollutant for US AQI
    #[serde(rename = "mainus")]
    pub main_us: String,
    #[serde(rename = "aqicn")]
    pub aqi_cn: u32,
    /// main pollutant for CN AQI
    #[serde(rename = "maincn")]
    pub main_cn: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Weather {
    #[serde(rename = "ts")]
    #[serde(with = "time::serde::iso8601")]
    pub timestamp: OffsetDateTime,
    #[serde(rename = "tp")]
    /// temperature in Celsius
    pub temp: i32,
    #[serde(rename = "pr")]
    /// atmospheric pressure in hPa
    pub pressure: i32,
    #[serde(rename = "hu")]
    /// humidity %
    pub humidity: u16,
    #[serde(rename = "ws")]
    /// wind speed (m/s)
    pub wind_speed: f32,
    #[serde(rename = "wd")]
    /// wind direction, as an angle of 360° (N=0, E=90, S=180, W=270)
    pub wind_direction: i16,
    #[serde(rename = "ic")]
    /// weather icon code, see below for icon index
    pub icon: Icon,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub enum Icon {
    #[serde(rename = "01d")]
    ClearDaySky,
    #[serde(rename = "01n")]
    ClearNightSky,
    #[serde(rename = "02d")]
    FewDayClouds,
    #[serde(rename = "02n")]
    FewNightClouds,
    #[serde(rename = "03d")]
    ScatteredClouds,
    #[serde(rename = "04d")]
    BrokenClouds,
    #[serde(rename = "09d")]
    ShowerRain,
    #[serde(rename = "10d")]
    DayRain,
    #[serde(rename = "10n")]
    NightRain,
    #[serde(rename = "11d")]
    Thunderstorm,
    #[serde(rename = "13d")]
    Snow,
    #[serde(rename = "50d")]
    Mist,
    #[serde(untagged)]
    Other(String),
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Error {
    Success,
    ///  when minute/monthly limit is reached.
    CallLimitReached,
    /// when API key is expired.
    ApiKeyExpired,
    /// returned when using wrong API key.
    IncorrectApiKey,
    /// when service is unable to locate IP address of request.
    IpLocationFailed,
    /// when there is no nearest station within specified radius.
    NoNearestStation,
    /// when call requests a feature that is not available in chosen subscription plan.
    FeatureNotAvailable,
    /// when more than 10 calls per second are made.
    TooManyRequests,
    #[serde(skip)]
    Io(std::io::Error),
    #[serde(skip)]
    Ureq(ureq::Error),
    #[serde(skip)]
    Serde(serde_json::Error),
}
impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use Error::*;
        match self {
            Success => unreachable!(),
            CallLimitReached => f.write_str("minute/monthly limit is reached"),
            ApiKeyExpired => f.write_str("API key is expired"),
            Error::Ureq(ureq::Error::StatusCode(403)) | IncorrectApiKey => {
                f.write_str("using wrong API key")
            }
            IpLocationFailed => f.write_str("service is unable to locate IP address of request"),
            NoNearestStation => f.write_str("there is no nearest station within specified radius"),
            FeatureNotAvailable => f.write_str(
                "call requests a feature that is not available in chosen subscription plan",
            ),
            TooManyRequests => f.write_str("more than 10 calls per second are made"),
            Error::Io(error) => write!(f, "{error}"),
            Error::Ureq(error) => write!(f, "{error}"),
            Error::Serde(error) => write!(f, "{error}"),
        }
    }
}
impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(match self {
            Error::Io(error) => error,
            Error::Ureq(error) => error,
            Error::Serde(error) => error,
            _ => return None,
        })
    }
}

/// Get pollution data for nearest city.
pub fn nearest(key: &str) -> std::result::Result<Data, Error> {
    let uri = format!("https://api.airvisual.com/v2/nearest_city?key={key}");
    let mut result = String::with_capacity(50);
    std::io::Read::read_to_string(
        &mut ureq::get(uri)
            .call()
            .map_err(Error::Ureq)?
            .body_mut()
            .as_reader(),
        &mut result,
    )
    .map_err(Error::Io)?;
    #[derive(Deserialize)]
    struct Status {
        status: Error,
    }
    let Status { status } = serde_json::from_str::<Status>(&result).map_err(Error::Serde)?;
    if !matches!(status, Error::Success) {
        return Err(status);
    }
    serde_json::from_str::<Dat>(&result)
        .map_err(Error::Serde)
        .map(|Dat { data }| data)
}

#[test]
fn x() {
    dbg!(nearest("294e67bb-404b-41b5-a73d-60bb71f361f2").unwrap());
}
