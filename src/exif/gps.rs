use chrono::{NaiveDate, NaiveTime};

use little_exif::exif_tag::ExifTag;
use little_exif::metadata::Metadata;
use little_exif::rational::uR64;

use super::utils::{
    F64, AllList, RemoveTag, 
    pick_v0, pick_v0_ur64, 
    pick_v04, pick_v03_ur64,
    some_string,
};

use crate::exif::rational::approx_frac;
use crate::{define_big_struct, define_enum, define_str_enum, define_struct};
use crate::components::utils::ShowValue;

define_big_struct!(
    GpsInfo,
    location_info: LocationInfo,
);

define_struct!(
    LocationInfo,
    fields: {
        gps_latitude_ref: bool,
        gps_latitude: DMS,
        gps_longitude_ref: bool,
        gps_longitude: DMS,
        gps_map_datum: String,
        gps_altitude_ref: GPSAltitudeRef,
        gps_altitude: F64,

        gps_satellites: String,
        gps_status: GPSStatus,
        gps_measure_mode: GPSMeasureMode,
        gps_dop: F64,

        gps_speed_ref: GPSSpeedRef,
        gps_speed: F64,
        gps_track_ref: NorthRef,
        gps_track: F64,
        gps_img_direction_ref: NorthRef,
        gps_img_direction: F64,

        gps_dest_latitude_ref: bool,
        gps_dest_latitude: DMS,
        gps_dest_longitude_ref: bool,
        gps_dest_longitude: DMS,
        gps_dest_bearing_ref: NorthRef,
        gps_dest_bearing: F64,
        gps_dest_distance_ref: GPSDestDistanceRef,
        gps_dest_distance: F64,

        gps_processing_method: Vec<u8>,
        gps_area_information: Vec<u8>,
        gps_date_stamp: NaiveDate,
        gps_time_stamp: NaiveTime,
        gps_differential: GPSDifferential,
        gps_h_positioning_error: F64,
        gps_version_id: [u8; 4],
        gps_info: u32,
    },
    tags: {
        gps_latitude_ref: (GPSLatitudeRef, gps_ref),
        gps_latitude: (GPSLatitude, DMS::from_vec),
        gps_longitude_ref: (GPSLongitudeRef, gps_ref),
        gps_longitude: (GPSLongitude, DMS::from_vec),
        gps_map_datum: (GPSMapDatum, some_string),
        gps_altitude_ref: (GPSAltitudeRef, |v: &Vec<u8>| Some(GPSAltitudeRef::from_u8_vec(v))),
        gps_altitude: (GPSAltitude, pick_v0_ur64),
        gps_satellites: (GPSSatellites, some_string),
        gps_status: (GPSStatus, |s: &str| Some(GPSStatus::from_str(s))),
        gps_measure_mode: (GPSMeasureMode, |s: &str| Some(GPSMeasureMode::from_str(s))),
        gps_dop: (GPSDOP, pick_v0_ur64),

        gps_speed_ref: (GPSSpeedRef, |s: &str| Some(GPSSpeedRef::from_str(s))),
        gps_speed: (GPSSpeed, pick_v0_ur64),
        gps_track_ref: (GPSTrackRef, |s: &str| Some(NorthRef::from_str(s))),
        gps_track: (GPSTrack, pick_v0_ur64),
        gps_img_direction_ref: (GPSImgDirectionRef, |s: &str| Some(NorthRef::from_str(s))),
        gps_img_direction: (GPSImgDirection, pick_v0_ur64),
        
        gps_dest_latitude_ref: (GPSDestLatitudeRef, gps_ref),
        gps_dest_latitude: (GPSDestLatitude, DMS::from_vec),
        gps_dest_longitude_ref: (GPSDestLongitudeRef, gps_ref),
        gps_dest_longitude: (GPSDestLongitude, DMS::from_vec),
        gps_dest_bearing_ref: (GPSDestBearingRef, |s: &str| Some(NorthRef::from_str(s))),
        gps_dest_bearing: (GPSDestBearing, pick_v0_ur64),
        gps_dest_distance_ref: (GPSDestDistanceRef, |s: &str| Some(GPSDestDistanceRef::from_str(s))),
        gps_dest_distance: (GPSDestDistance, pick_v0_ur64),

        // gps_processing_method: 未実装
        gps_processing_method: (GPSProcessingMethod, |v: &[u8]| Some(v.to_vec())),
        // gps_area_information: 未実装
        gps_area_information: (GPSAreaInformation, |v: &[u8]| Some(v.to_vec())),
        gps_date_stamp: (GPSDateStamp, parse_date),
        gps_time_stamp: (GPSTimeStamp, parse_time),
        gps_differential: (GPSDifferential, |v: &[u16]| Some(GPSDifferential::from_vec(v))),
        gps_h_positioning_error: (GPSHPositioningError, pick_v0_ur64),
        gps_version_id: (GPSVersionID, pick_v04),
        gps_info: (GPSInfo, pick_v0)
        ;
    }
);

pub fn gps_ref(s: &str) -> Option<bool> {
    Some(s == "N" || s == "E")
}

pub fn parse_date(s: &str) -> Option<NaiveDate> {
    match NaiveDate::parse_from_str(s, "%Y:%m:%d") {
        Ok(nd) => Some(nd),
        Err(_) => None,
    }
}

pub fn parse_time(v: &[uR64]) -> Option<NaiveTime> {
    let a = match pick_v03_ur64(v) {
        Some(a) => a,
        None => { return None; }
    };
    let hour = a[0].value() as u32;
    let min = a[1].value() as u32;
    let sec = a[2].value() as u32;
    let milli = ((a[2].value() - sec as f64) * 1000.0) as u32;
    NaiveTime::from_hms_milli_opt(hour, min, sec, milli)
}

#[derive(Clone, PartialEq)]
pub struct DMS {
    degree: u8,
    minute: u8,
    second: F64,
}

impl DMS {
    pub fn from_vec(v: &[uR64]) -> Option<Self> {
        if v.len() != 3 { return None; }
        let degree_f = F64::from_ur64(&v[0]).value();
        let minute_f = F64::from_ur64(&v[1]).value();
        let second = F64::from_ur64(&v[2]);
        let degree = if degree_f < 0.0 || degree_f > 180.0 {
            return None;
        } else {
            degree_f as u8
        };
        let minute = if minute_f < 0.0 || minute_f >= 60.0 {
            return None;
        } else {
            minute_f as u8
        };
        Some(Self { degree, minute, second })
    }

    pub fn to_vec(&self) -> Vec<uR64> {
        vec![
            uR64 { nominator: self.degree as u32, denominator: 1 },
            uR64 { nominator: self.minute as u32, denominator: 1 },
            uR64 {
                nominator: (self.second.value() * self.second.den() as f64) as u32, 
                denominator: self.second.den()
            }
        ]
    }

    pub fn from_f64(f: f64, den: Option<u32>) -> Option<Self> {
        if f < -180.0 || f > 180.0 { return None; }
        let mut value = f.abs();
        let degree = value as u8;
        value = (value - degree as f64) * 60.0;
        let minute = value as u8;
        value = (value - minute as f64) * 60.0;
        let (value, den) = match den {
            Some(den) => (value, den),
            None => {
                let (value, _, den) = approx_frac(value).unwrap();
                (value, den as u32)
            }
        };
        let second = F64::new(value, den);
        Some(Self { degree, minute, second })
    }

    pub fn to_f64(&self) -> f64 {
        self.degree as f64 + self.minute as f64 / 60.0 + self.second.value() / 3600.0
    }
}

define_enum!(
    GPSAltitudeRef {
        AboveSeaLevel = 0,
        BelowSeaLevel = 1,
        PositiveSeaLevel = 2,
        NegativeSeaLevel = 3,
    }, 255
);

define_str_enum!(
    GPSStatus {
        MeasurementActive = "A",
        MeasurementVoid = "V",
    }, MeasurementVoid
);

define_str_enum!(
    GPSMeasureMode {
        TwoDimensionalMeasurement = "2",
        ThreeDimensionalMeasurement = "3",
    }, TwoDimensionalMeasurement
);

define_str_enum!(
    GPSSpeedRef {
        km_h = "K",
        mph = "M",
        knots = "N",
    }, km_h
);

define_str_enum!(
    NorthRef {
        MagneticNorth = "M",
        TrueNorth = "T",
    }, MagneticNorth
);

define_str_enum!(
    GPSDestDistanceRef {
        Kilimeters = "K",
        Miles = "M",
        NauticalMiles = "N",
    }, Kilimeters
);

define_enum!(
    GPSDifferential {
        NoCorrection = 0,
        DifferentialCorrected = 1,
    }, 255
);