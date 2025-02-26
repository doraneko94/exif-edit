use std::fmt;

use little_exif::ifd::ExifTagGroup;
use little_exif::metadata::Metadata;
use little_exif::exif_tag::ExifTag;
use little_exif::rational::{iR64, uR64};
use chrono::NaiveDateTime;

#[derive(Debug)]
enum GPSLatitudeRef {
    N, S
}

#[derive(Debug)]
enum GPSLongitudeRef {
    E, W
}

#[derive(Debug)]
struct Coordinate {
    degree: uR64,
    minute: uR64,
    second: uR64,
}

impl Coordinate {
    pub fn new(degree: uR64, minute: uR64, second: uR64) -> Self {
        Self { degree, minute, second }
    }

    pub fn from_vec(v: &[uR64]) -> Result<Self, ()> {
        if v.len() != 3 { return Err(()); }
        let degree = v[0].clone();
        let minute = v[1].clone();
        let second = v[2].clone();
        Ok(Self::new(degree, minute, second))
    }

    pub fn from_f64(value: f64) -> Self {
        let mut value = value;
        let degree_nom = value as u32;
        let degree = uR64 { nominator: degree_nom, denominator: 1 };
        value -= degree_nom as f64;
        let minute_nom = (value * 60.0) as u32;
        let minute = uR64 { nominator: minute_nom, denominator: 1 };
        value -= minute_nom as f64 / 60.0;
        let second_nom = (value * 360000.0) as u32;
        let second = uR64 { nominator: second_nom, denominator: 100 };
        Self::new(degree, minute, second)
    }

    pub fn to_f64(&self) -> f64 {
        self.degree.nominator as f64 / self.degree.denominator as f64
        + self.minute.nominator as f64 /  self.minute.denominator as f64 / 60.0
        + self.second.nominator as f64 / self.second.denominator as f64 / 3600.0
    }
}

impl fmt::Display for Coordinate {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f, "{}°{}'{}\"",
            (self.degree.nominator as f64 / self.degree.denominator as f64) as u32,
            (self.minute.nominator as f64 / self.minute.denominator as f64) as u32,
            self.second.nominator as f64 / self.second.denominator as f64
        )
    }
}

pub struct ExifEditData {
    metadata: Metadata,

    // Core Tags
    strip_offsets: Option<(Vec<u32>, Vec<Vec<u8>>)>,
    strip_byte_counts: Option<Vec<u32>>,
    thumbnail_offset: Option<(Vec<u32>, Vec<u8>)>,
    thumbnail_length: Option<Vec<u32>>,
    image_width: Option<Vec<u32>>,
    image_height: Option<Vec<u32>>,
    orientation: Option<Vec<u16>>,

    // Edit Tags
    date_time_original: Option<NaiveDateTime>,
    create_date: Option<NaiveDateTime>,
    modify_date: Option<NaiveDateTime>,
    gps_latitude_ref: Option<GPSLatitudeRef>,
    gps_latitude: Option<Coordinate>,
    gps_longitude_ref: Option<GPSLongitudeRef>,
    gps_longitude: Option<Coordinate>,
    gps_altitude_ref: Option<u8>,
    gps_altitude: Option<uR64>,
    
    pub others: Vec<String>,
}

impl ExifEditData {
    pub fn new(metadata: &Metadata) -> Self {
        let mut ret = Self {
            metadata: metadata.clone(),

            strip_offsets: None, strip_byte_counts: None,
            thumbnail_offset: None, thumbnail_length: None,
            image_width: None, image_height: None,
            orientation: None,

            date_time_original: None, create_date: None, modify_date: None,
            gps_latitude_ref: None, gps_latitude: None,
            gps_longitude_ref: None, gps_longitude: None,
            gps_altitude_ref: None, gps_altitude: None,
            others: Vec::new()
        };
        
        for ifd in metadata.get_ifds() {
            for tag in ifd.get_tags() {
                match tag {
                    // Core Tags
                    ExifTag::StripOffsets(v0, v1) => {
                        ret.strip_offsets = Some((v0.clone(), v1.clone()));
                    }
                    ExifTag::StripByteCounts(v) => { ret.strip_byte_counts = Some(v.clone()); }
                    ExifTag::ThumbnailOffset(v0, v1) => {
                        ret.thumbnail_offset = Some((v0.clone(), v1.clone()));
                    }
                    ExifTag::ThumbnailLength(v) => { ret.thumbnail_length = Some(v.clone()); }
                    ExifTag::ImageWidth(v) => { ret.image_width = Some(v.clone()); }
                    ExifTag::ImageHeight(v) => { ret.image_height = Some(v.clone()); }
                    ExifTag::Orientation(v) => { ret.orientation = Some(v.clone()); }

                    // Edit Tags
                    ExifTag::DateTimeOriginal(s) => { ret.date_time_original = parse_datetime(s); }
                    ExifTag::CreateDate(s) => { ret.create_date = parse_datetime(s); }
                    ExifTag::ModifyDate(s) => { ret.modify_date = parse_datetime(s); }
                    ExifTag::GPSLatitudeRef(s) => {
                        if s == "N" { ret.gps_latitude_ref = Some(GPSLatitudeRef::N); }
                        else if s == "S" { ret.gps_latitude_ref = Some(GPSLatitudeRef::S); }
                        else { ret.gps_latitude_ref = None; }
                    }
                    ExifTag::GPSLatitude(v) => {
                        ret.gps_latitude = match Coordinate::from_vec(&v) {
                            Ok(coord) => Some(coord),
                            Err(_) => None
                        };
                    }
                    ExifTag::GPSLongitudeRef(s) => {
                        if s == "E" { ret.gps_longitude_ref = Some(GPSLongitudeRef::E); }
                        else if s == "W" { ret.gps_longitude_ref = Some(GPSLongitudeRef::W); }
                        else { ret.gps_longitude_ref = None; }
                    }
                    ExifTag::GPSLongitude(v) => {
                        ret.gps_longitude = match Coordinate::from_vec(&v) {
                            Ok(coord) => Some(coord),
                            Err(_) => None
                        };
                    }
                    ExifTag::GPSAltitudeRef(v) => {
                        ret.gps_altitude_ref = pick_v0(v);
                    }
                    ExifTag::GPSAltitude(v) => {
                        ret.gps_altitude = pick_v0(v);
                    }

                    ExifTag::GPSVersionID(v) => { ret.others.push(format!("GPSVersionID: {}", string_int(v))); }
                    ExifTag::GPSTimeStamp(v) => { ret.others.push(format!("GPSTimeStamp: {}", string_rational(v))); }
                    ExifTag::GPSSatellites(s) => { ret.others.push(format!("GPSSatellites: {}", s)); }
                    ExifTag::GPSStatus(s) => { ret.others.push(format!("GPSStatus: {}", s)); }
                    ExifTag::GPSMeasureMode(s) => { ret.others.push(format!("GPSMeasureMode: {}", s)); }
                    ExifTag::GPSDOP(v) => { ret.others.push(format!("GPSDOP: {}", string_rational(v))); }
                    ExifTag::GPSSpeedRef(s) => { ret.others.push(format!("GPSSpeedRef: {}", s)); }
                    ExifTag::GPSSpeed(v) => { ret.others.push(format!("GPSSpeed: {}", string_rational(v))); }
                    ExifTag::GPSTrackRef(s) => { ret.others.push(format!("GPSTrackRef: {}", s)); }
                    ExifTag::GPSTrack(v) => { ret.others.push(format!("GPSTrack: {}", string_rational(v))); }
                    ExifTag::GPSImgDirectionRef(s) => { ret.others.push(format!("GPSImgDirectionRef: {}", s)); }
                    ExifTag::GPSImgDirection(v) => { ret.others.push(format!("GPSImgDirection: {}", string_rational(v))); }
                    ExifTag::GPSMapDatum(s) => { ret.others.push(format!("GPSMapDatum: {}", s)); }
                    ExifTag::GPSDestLatitudeRef(s) => { ret.others.push(format!("GPSDestLatitudeRef: {}", s)); }
                    ExifTag::GPSDestLatitude(v) => { ret.others.push(format!("GPSDestLatitude: {}", string_rational(v))); }
                    ExifTag::GPSDestLongitudeRef(s) => { ret.others.push(format!("GPSDestLongitudeRef: {}", s)); }
                    ExifTag::GPSDestLongitude(v) => { ret.others.push(format!("GPSDestLongitude: {}", string_rational(v))); }
                    ExifTag::GPSDestBearingRef(s) => { ret.others.push(format!("GPSDestBearingRef: {}", s)); }
                    ExifTag::GPSDestBearing(v) => { ret.others.push(format!("GPSDestBearing: {}", string_rational(v))); }
                    ExifTag::GPSDestDistanceRef(s) => { ret.others.push(format!("GPSDestDistanceRef: {}", s)); }
                    ExifTag::GPSDestDistance(v) => { ret.others.push(format!("GPSDestDistance: {}", string_rational(v))); }
                    ExifTag::GPSProcessingMethod(v) => { ret.others.push(format!("GPSProcessingMethod: {:?}", v)); }
                    ExifTag::GPSAreaInformation(v) => { ret.others.push(format!("GPSAreaInformation: {:?}", v)); }
                    ExifTag::GPSDateStamp(s) => { ret.others.push(format!("GPSDateStamp: {}", s)); }
                    ExifTag::GPSDifferential(v) => { ret.others.push(format!("GPSDifferential: {}", string_int(v))); }
                    ExifTag::GPSHPositioningError(v) => { ret.others.push(format!("GPSHPositioningError: {}", string_rational(v))); }
                    ExifTag::InteroperabilityIndex(s) => { ret.others.push(format!("InteroperabilityIndex: {}", s)); }
                    ExifTag::InteroperabilityVersion(v) => { ret.others.push(format!("InteroperabilityVersion: {:?}", v)); }
                    ExifTag::BitsPerSample(v) => { ret.others.push(format!("BitsPerSample: {}", string_int(v))); }
                    ExifTag::Compression(v) => { ret.others.push(format!("Compression: {}", string_int(v))); }
                    ExifTag::PhotometricInterpretation(v) => { ret.others.push(format!("PhotometricInterpretation: {}", string_int(v))); }
                    ExifTag::CellWidth(v) => { ret.others.push(format!("CellWidth: {}", string_int(v))); }
                    ExifTag::CellHeight(v) => { ret.others.push(format!("CellHeight: {}", string_int(v))); }
                    ExifTag::ImageDescription(s) => { ret.others.push(format!("ImageDescription: {}", s)); }
                    ExifTag::Make(s) => { ret.others.push(format!("Make: {}", s)); }
                    ExifTag::Model(s) => { ret.others.push(format!("Model: {}", s)); }
                    ExifTag::SamplesPerPixel(v) => { ret.others.push(format!("SamplesPerPixel: {}", string_int(v))); }
                    ExifTag::RowsPerStrip(v) => { ret.others.push(format!("RowsPerStrip: {}", string_int(v))); }
                    ExifTag::XResolution(v) => { ret.others.push(format!("XResolution: {}", string_rational(v))); }
                    ExifTag::YResolution(v) => { ret.others.push(format!("YResolution: {}", string_rational(v))); }
                    ExifTag::PlanarConfiguration(v) => { ret.others.push(format!("PlanarConfiguretion: {}", string_int(v))); }
                    ExifTag::ResolutionUnit(v) => { ret.others.push(format!("ResolutionUnit: {}", string_int(v))); }
                    ExifTag::TransferFunction(v) => { ret.others.push(format!("TransferFunction: {}", string_int(v))); }
                    ExifTag::Software(s) => { ret.others.push(format!("Software: {}", s)); }
                    ExifTag::Artist(s) => { ret.others.push(format!("Artist: {}", s)); }
                    ExifTag::WhitePoint(v) => { ret.others.push(format!("WhitePoint: {}", string_rational(v))); }
                    ExifTag::PrimaryChromaticities(v) => { ret.others.push(format!("PrimaryChromaticities: {}", string_rational(v))); }
                    ExifTag::ColorMap(v) => { ret.others.push(format!("ColorMap: {}", string_int(v))); }
                    ExifTag::YCbCrCoefficients(v) => { ret.others.push(format!("YCbCrCoefficients: {}", string_rational(v))); }
                    ExifTag::YCbCrSubSampling(v) => { ret.others.push(format!("YCbCrSubSampling: {}", string_int(v))); }
                    ExifTag::YCbCrPositioning(v) => { ret.others.push(format!("YCbCrPositioning: {}", string_int(v))); }
                    ExifTag::ReferenceBlackWhite(v) => { ret.others.push(format!("ReferenceBlackWhite: {}", string_rational(v))); }
                    ExifTag::Copyright(s) => { ret.others.push(format!("Copyright: {}", s)); }
                    ExifTag::ExposureTime(v) => { ret.others.push(format!("ExposureTime: {}", string_rational(v))); }
                    ExifTag::FNumber(v) => { ret.others.push(format!("FNumber: {}", string_rational(v))); }
                    ExifTag::ExifOffset(v) => { ret.others.push(format!("ExifOffset: {}", string_int(v))); }
                    ExifTag::ExposureProgram(v) => { ret.others.push(format!("ExposureProgram: {}", string_int(v))); }
                    ExifTag::SpectralSensitivity(s) => { ret.others.push(format!("SpectralSensitivity: {}", s)); }
                    ExifTag::GPSInfo(v) => { ret.others.push(format!("GPSInfo: {}", string_int(v))); }
                    ExifTag::ISO(v) => { ret.others.push(format!("ISO: {}", string_int(v))); }
                    ExifTag::OECF(v) => { ret.others.push(format!("OECF: {:?}", v)); }
                    ExifTag::SensitivityType(v) => { ret.others.push(format!("SensitivityType: {}", string_int(v))); }
                    ExifTag::StandardOutputSensitivity(v) => { ret.others.push(format!("StandardOutputSensitivity: {}", string_int(v))); }
                    ExifTag::RecommendedExposureIndex(v) => { ret.others.push(format!("RecommendedExposureIndex: {}", string_int(v))); }
                    ExifTag::ISOSpeed(v) => { ret.others.push(format!("ISOSpeed: {}", string_int(v))); }
                    ExifTag::ISOSpeedLatitudeyyy(v) => { ret.others.push(format!("ISOSpeedLatitudeyyy: {}", string_int(v))); }
                    ExifTag::ISOSpeedLatitudezzz(v) => { ret.others.push(format!("ISOSpeedLatitudezzz: {}", string_int(v))); }
                    ExifTag::ExifVersion(v) => { ret.others.push(format!("ExifVersion: {:?}", v)); }
                    ExifTag::OffsetTime(s) => { ret.others.push(format!("OffsetTime: {}", s)); }
                    ExifTag::OffsetTimeOriginal(s) => { ret.others.push(format!("OffsetTimeOriginal: {}", s)); }
                    ExifTag::OffsetTimeDigitized(s) => { ret.others.push(format!("OffsetTimeDigitized: {}", s)); }
                    ExifTag::ComponentsConfiguration(v) => { ret.others.push(format!("ComponentsConfiguretion: {:?}", v)); }
                    ExifTag::CompressedBitsPerPixel(v) => { ret.others.push(format!("CompressedBitsPerPixel: {}", string_rational(v))); }
                    ExifTag::ShutterSpeedValue(v) => { ret.others.push(format!("ShutterSpeedValue: {}", string_rational(v))); }
                    ExifTag::ApertureValue(v) => { ret.others.push(format!("ApertureValue: {}", string_rational(v))); }
                    ExifTag::BrightnessValue(v) => { ret.others.push(format!("BrightnessValue: {}", string_rational(v))); }
                    ExifTag::ExposureCompensation(v) => { ret.others.push(format!("ExposureCompensation: {}", string_rational(v))); }
                    ExifTag::MaxApertureValue(v) => { ret.others.push(format!("MaxApertureValue: {}", string_rational(v))); }
                    ExifTag::SubjectDistance(v) => { ret.others.push(format!("SubjectDistance: {}", string_rational(v))); }
                    ExifTag::MeteringMode(v) => { ret.others.push(format!("MeteringMode: {}", string_int(v))); }
                    ExifTag::LightSource(v) => { ret.others.push(format!("LightSource: {}", string_int(v))); }
                    ExifTag::Flash(v) => { ret.others.push(format!("Flash: {}", string_int(v))); }
                    ExifTag::FocalLength(v) => { ret.others.push(format!("FocalLength: {}", string_rational(v))); }
                    ExifTag::SubjectArea(v) => { ret.others.push(format!("SubjectArea: {}", string_int(v))); }
                    ExifTag::MakerNote(v) => { ret.others.push(format!("MakerNote: {:?}", v)); }
                    ExifTag::UserComment(v) => { ret.others.push(format!("UserComment: {:?}", v)); }
                    ExifTag::SubSecTime(s) => { ret.others.push(format!("SubSecTime: {}", s)); }
                    ExifTag::SubSecTimeOriginal(s) => { ret.others.push(format!("SubSecTimeOriginal: {}", s)); }
                    ExifTag::SubSecTimeDigitized(s) => { ret.others.push(format!("SubSecTimeDigitized: {}", s)); }
                    ExifTag::AmbientTemperature(v) => { ret.others.push(format!("AmbientTemperature: {}", string_rational(v))); }
                    ExifTag::Humidity(v) => { ret.others.push(format!("Humidity: {}", string_rational(v))); }
                    ExifTag::Pressure(v) => { ret.others.push(format!("Pressure: {}", string_rational(v))); }
                    ExifTag::WaterDepth(v) => { ret.others.push(format!("WaterDepth: {}", string_rational(v))); }
                    ExifTag::Acceleration(v) => { ret.others.push(format!("Acceleration: {}", string_rational(v))); }
                    ExifTag::CameraElevationAngle(v) => { ret.others.push(format!("CameraElevationAngle: {}", string_rational(v))); }
                    ExifTag::FlashpixVersion(v) => { ret.others.push(format!("FlashpixVersion: {:?}", v)); }
                    ExifTag::ColorSpace(v) => { ret.others.push(format!("ColorSpace: {}", string_int(v))); }
                    ExifTag::ExifImageWidth(v) => { ret.others.push(format!("ExifImageWidth: {}", string_int(v))); }
                    ExifTag::ExifImageHeight(v) => { ret.others.push(format!("ExifImageHeight: {}", string_int(v))); }
                    ExifTag::RelatedSoundFile(s) => { ret.others.push(format!("RelatedSoundFile: {}", s)); }
                    ExifTag::InteropOffset(v) => { ret.others.push(format!("InteropOffset: {}", string_int(v))); }
                    ExifTag::FlashEnergy(v) => { ret.others.push(format!("FlashEnergy: {}", string_rational(v))); }
                    ExifTag::SpatialFrequencyResponse(v) => { ret.others.push(format!("SpatialFrequencyResponse: {}", string_int(v))); }
                    ExifTag::FocalPlaneXResolution(v) => { ret.others.push(format!("FocalPlaneXResolution: {}", string_rational(v))); }
                    ExifTag::FocalPlaneYResolution(v) => { ret.others.push(format!("FocalPlaneYResolution: {}", string_rational(v))); }
                    ExifTag::FocalPlaneResolutionUnit(v) => { ret.others.push(format!("FocalPlaneResolutionUnit: {}", string_int(v))); }
                    ExifTag::SubjectLocation(v) => { ret.others.push(format!("SubjectLocation: {}", string_int(v))); }
                    ExifTag::ExposureIndex(v) => { ret.others.push(format!("ExposureIndex: {}", string_rational(v))); }
                    ExifTag::SensingMethod(v) => { ret.others.push(format!("SensingMethod: {}", string_int(v))); }
                    ExifTag::FileSource(v) => { ret.others.push(format!("FileSource: {:?}", v)); }
                    ExifTag::SceneType(v) => { ret.others.push(format!("SceneType: {:?}", v)); }
                    ExifTag::CFAPattern(v) => { ret.others.push(format!("CFAPattern: {:?}", v)); }
                    ExifTag::CustomRendered(v) => { ret.others.push(format!("CustomRendered: {}", string_int(v))); }
                    ExifTag::ExposureMode(v) => { ret.others.push(format!("ExposureMode: {}", string_int(v))); }
                    ExifTag::WhiteBalance(v) => { ret.others.push(format!("WhiteBalance: {}", string_int(v))); }
                    ExifTag::DigitalZoomRatio(v) => { ret.others.push(format!("DigitalZoomRatio: {}", string_rational(v))); }
                    ExifTag::FocalLengthIn35mmFormat(v) => { ret.others.push(format!("FocalLengthIn35mmFormat: {}", string_int(v))); }
                    ExifTag::SceneCaptureType(v) => { ret.others.push(format!("SceneCaptureType: {}", string_int(v))); }
                    ExifTag::GainControl(v) => { ret.others.push(format!("GainControl: {}", string_int(v))); }
                    ExifTag::Contrast(v) => { ret.others.push(format!("Contrast: {}", string_int(v))); }
                    ExifTag::Saturation(v) => { ret.others.push(format!("Saturation: {}", string_int(v))); }
                    ExifTag::Sharpness(v) => { ret.others.push(format!("Sharpness: {}", string_int(v))); }
                    ExifTag::DeviceSettingDescription(v) => { ret.others.push(format!("DeviceSettingDescription: {:?}", v)); }
                    ExifTag::SubjectDistanceRange(v) => { ret.others.push(format!("SubjectDistanceRange: {}", string_int(v))); }
                    ExifTag::ImageUniqueID(s) => { ret.others.push(format!("ImageUniqueID: {}", s)); }
                    ExifTag::OwnerName(s) => { ret.others.push(format!("OwnerName: {}", s)); }
                    ExifTag::SerialNumber(s) => { ret.others.push(format!("SerialNumber: {}", s)); }
                    ExifTag::LensInfo(v) => { ret.others.push(format!("LensInfo: {}", string_rational(v))); }
                    ExifTag::LensMake(s) => { ret.others.push(format!("LensMake: {}", s)); }
                    ExifTag::LensModel(s) => { ret.others.push(format!("LensModel: {}", s)); }
                    ExifTag::LensSerialNumber(s) => { ret.others.push(format!("LensSerialNumber: {}", s)); }
                    ExifTag::CompositeImage(v) => { ret.others.push(format!("CompositeImage: {}", string_int(v))); }
                    ExifTag::CompositeImageCount(v) => { ret.others.push(format!("CompositeImageCount: {}", string_int(v))); }
                    ExifTag::CompositeImageExposureTimes(v) => { ret.others.push(format!("CompositeImageExposureTimes: {:?}", v)); }
                    ExifTag::Gamma(v) => { ret.others.push(format!("Gamma: {}", string_rational(v))); }
                    
                    ExifTag::UnknownINT8U(v, hex, group) => { ret.others.push(format!("{}: {}", unknown_string_core(*hex, group.clone()), string_int(v))); }
                    ExifTag::UnknownSTRING(s, hex, group) => { ret.others.push(format!("{}: {}", unknown_string_core(*hex, group.clone()), s)); }
                    ExifTag::UnknownINT16U(v, hex, group) => { ret.others.push(format!("{}: {}", unknown_string_core(*hex, group.clone()), string_int(v))); }
                    ExifTag::UnknownINT32U(v, hex, group) => { ret.others.push(format!("{}: {}", unknown_string_core(*hex, group.clone()), string_int(v))); }
                    ExifTag::UnknownRATIONAL64U(v, hex, group) => { ret.others.push(format!("{}: {}", unknown_string_core(*hex, group.clone()), string_rational(v))); }
                    ExifTag::UnknownINT8S(v, hex, group) => { ret.others.push(format!("{}: {}", unknown_string_core(*hex, group.clone()), string_int(v))); }
                    ExifTag::UnknownUNDEF(v, hex, group) => { ret.others.push(format!("{}: {:?}", unknown_string_core(*hex, group.clone()), v)); }
                    ExifTag::UnknownINT16S(v, hex, group) => { ret.others.push(format!("{}: {}", unknown_string_core(*hex, group.clone()), string_int(v))); }
                    ExifTag::UnknownINT32S(v, hex, group) => { ret.others.push(format!("{}: {}", unknown_string_core(*hex, group.clone()), string_int(v))); }
                    ExifTag::UnknownRATIONAL64S(v, hex, group) => { ret.others.push(format!("{}: {}", unknown_string_core(*hex, group.clone()), string_rational(v))); }
                    ExifTag::UnknownFLOAT(v, hex, group) => { ret.others.push(format!("{}: {}", unknown_string_core(*hex, group.clone()), string_int(v))); }
                    ExifTag::UnknownDOUBLE(v, hex, group) => { ret.others.push(format!("{}: {}", unknown_string_core(*hex, group.clone()), string_int(v))); }
                }
            }
        }
        ret
    }
}

fn parse_datetime(s: &str) -> Option<NaiveDateTime> {
    match NaiveDateTime::parse_from_str(s, "%Y:%m:%d %H:%M:%S") {
        Ok(ndt) => Some(ndt),
        Err(_) => None,
    }
}

fn pick_v0<T: Clone>(v: &[T]) -> Option<T> {
    if v.len() != 1 { None }
    else { Some(v[0].clone()) }
}

fn string_int<T: fmt::Debug + fmt::Display>(v: &[T]) -> String {
    let length = v.len();
    if length == 0 { "None".to_string() }
    else if length == 1 { format!("{}", v[0]) }
    else { format!("{:?}", v) }
}

fn string_rational<S, T: Rational<S>>(v: &[T]) -> String {
    let length = v.len();
    if length == 0 { "None".to_string() }
    else if length == 1 { format!("{}", v[0].float()) }
    else {
        format!("{:?}", v.iter().map(|vi| vi.float()).collect::<Vec<f32>>()) }
}

pub fn unknown_string_core(hex: u16, group: ExifTagGroup) -> String {
    "Unknown(".to_string()
    + match group {
        ExifTagGroup::GENERIC => "Generic",
        ExifTagGroup::EXIF => "Exif",
        ExifTagGroup::INTEROP => "Interop",
        ExifTagGroup::GPS => "GPS",
    } + &format!(", hex:0x{:x})", hex)
}

trait Rational<T> {
    fn float(&self) -> f32;
}

impl Rational<u32> for uR64 {
    fn float(&self) -> f32 { self.nominator as f32 / self.denominator as f32 }
}

impl Rational<i32> for iR64 {
    fn float(&self) -> f32 { self.nominator as f32 / self.denominator as f32 }
}