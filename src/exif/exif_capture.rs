use chrono::NaiveDateTime;
use little_exif::exif_tag::ExifTag;
use little_exif::metadata::Metadata;

use super::utils::{
    F64, AllList, RemoveTag, VersionAscii,
    some_string,
    pick_v0, pick_v0_ir64, pick_v0_ur64,
    pick_v02
};
use crate::{define_big_struct, define_enum, define_struct};
use crate::components::utils::ShowValue;

define_big_struct!(
    ExifCaptureInfo,
    time_info: TimeInfo,
    optic_info: OpticInfo,
    exposure_settings: ExposureSettings,
    sensitivity_info: SensitivityInfo,
    encoding_metadata: EncodingMetadata,
    identifier_info: IdentifierInfo,
    composite_metadata: CompositeMetadata,
);

define_struct!(
    TimeInfo,
    fields: {
        date_time_original: NaiveDateTime,
        offset_time_original: TimeOffset,
        sub_sec_time_original: u16,

        create_date: NaiveDateTime,
        offset_time_digitized: TimeOffset,
        sub_sec_time_digitized: u16,

        modify_date: NaiveDateTime,
        offset_time: TimeOffset,
        sub_sec_time: u16,
    },
    tags: {
        date_time_original: (DateTimeOriginal, parse_datetime),
        offset_time_original: (OffsetTimeOriginal, TimeOffset::from_str),
        sub_sec_time_original: (SubSecTimeOriginal, |s: &str| match s.parse::<u16>() { Ok(u) => Some(u), Err(_) => None }),

        create_date: (CreateDate, parse_datetime),
        offset_time_digitized: (OffsetTimeDigitized, TimeOffset::from_str),
        sub_sec_time_digitized: (SubSecTimeDigitized, |s: &str| match s.parse::<u16>() { Ok(u) => Some(u), Err(_) => None }),

        modify_date: (ModifyDate, parse_datetime),
        offset_time: (OffsetTime, TimeOffset::from_str),
        sub_sec_time: (SubSecTime, |s: &str| match s.parse::<u16>() { Ok(u) => Some(u), Err(_) => None });
    }
);

define_struct!(
    OpticInfo,
    fields: {
        lens_make: String,
        lens_model: String,
        lens_serial_number: String,
        max_aperture_value: F64,
    },
    tags: {
        lens_make: (LensMake, some_string),
        lens_model: (LensModel, some_string),
        lens_serial_number: (LensSerialNumber, some_string),
        max_aperture_value: (MaxApertureValue, pick_v0_ur64);
    }
);

define_struct!(
    ExposureSettings,
    fields: {
        exposure_program: ExposureProgram,
        exposure_mode: ExposureMode,

        exposure_time: F64,
        shutter_speed_value: F64,

        f_number: F64,
        aperture_value: F64,

        exposure_compensation: F64,
        brightness_value: F64,

        metering_mode: MeteringMode,
        light_source: LightSource,
        flash: Flash,

        focal_length: F64,
        subject_area: Vec<u16>,
        subject_location: [u16; 2],
    },
    tags: {
        exposure_program: (ExposureProgram, |v: &[u16]| Some(ExposureProgram::from_vec(v))),
        exposure_mode: (ExposureMode, |v: &[u16]| Some(ExposureMode::from_vec(v))),

        exposure_time: (ExposureTime, pick_v0_ur64),
        shutter_speed_value: (ShutterSpeedValue, pick_v0_ir64),

        f_number: (FNumber, pick_v0_ur64),
        aperture_value: (ApertureValue, pick_v0_ur64),

        exposure_compensation: (ExposureCompensation, pick_v0_ir64),
        brightness_value: (BrightnessValue, pick_v0_ir64),

        metering_mode: (MeteringMode, |v: &[u16]| Some(MeteringMode::from_vec(v))),
        light_source: (LightSource, |v: &[u16]| Some(LightSource::from_vec(v))),
        flash: (Flash, |v: &[u16]| Some(Flash::from_vec(v))),

        focal_length: (FocalLength, pick_v0_ur64),
        subject_area: (SubjectArea, |v: &[u16]| Some(v.to_vec())),
        subject_location: (SubjectLocation, pick_v02);
    }
);

define_struct!(
    SensitivityInfo,
    fields: {
        sensitivity_type: SensitivityType,

        iso: Vec<u16>,
        iso_speed: u32,

        standard_output_sensitivity: u32,
        recommended_exposure_index: u32,
        exposure_index: F64,

        iso_speed_latitude_yyy: u32,
        iso_speed_latitude_zzz: u32,
    },
    tags: {
        sensitivity_type: (SensitivityType, |v: &[u16]| Some(SensitivityType::from_vec(v))),

        iso: (ISO, |v: &[u16]| Some(v.to_vec())),
        iso_speed: (ISOSpeed, pick_v0),

        standard_output_sensitivity: (StandardOutputSensitivity, pick_v0),
        recommended_exposure_index: (RecommendedExposureIndex, pick_v0),
        exposure_index: (ExposureIndex, pick_v0_ur64),

        iso_speed_latitude_yyy: (ISOSpeedLatitudeyyy, pick_v0),
        iso_speed_latitude_zzz: (ISOSpeedLatitudezzz, pick_v0);
    }
);

define_struct!(
    EncodingMetadata,
    fields: {
        exif_version: String,
        flashpix_version: String,
        exif_offset: u32,
        components_configuration: [ComponentsConfiguration; 4],
        compressed_bits_per_pixel: F64,

        sensing_method: SensingMethod,
        file_source: FileSource,
        scene_type: SceneType,
        cfa_pattern: CFAPattern,

        ambient_temperature: F64,
        humidity: F64,
        pressure: F64,
        water_depth: F64,
        acceleration: F64,
        camera_elevation_angle: F64,

        spectral_sensitivity: String,
        oecf: Vec<u8>,
        subject_distance: F64,
        subject_distance_range: u16,
        flash_energy: F64,
        spatial_frequency_response: Vec<u16>,

        focal_plane_x_resolution: F64,
        focal_plane_y_resolution: F64,
        focal_plane_resolution_unit: FocalPlaneResolutionUnit,

        white_balance: WhiteBalance,
        digital_zoom_ratio: F64,
        focal_length_in_35mm_format: u16,
        scene_capture_type: SceneCaptureType,
        gain_control: GainControl,
        contrast: Contrast,
        saturation: Saturation,
        sharpness: Sharpness,
        custom_rendered: CustomRendered,
        device_setting_description: Vec<u8>,
        gamma: F64,

        related_sound_file: String,
    },
    tags: {
        exif_version: (ExifVersion, |v: &Vec<u8>| v.to_string()),
        flashpix_version: (FlashpixVersion, |v: &Vec<u8>| v.to_string()),
        exif_offset: (ExifOffset, pick_v0),
        components_configuration: (ComponentsConfiguration, load_components_configuration),
        compressed_bits_per_pixel: (CompressedBitsPerPixel, pick_v0_ur64),

        sensing_method: (SensingMethod, |v: &[u16]| Some(SensingMethod::from_vec(v))),
        file_source: (FileSource, |v: &[u8]| Some(FileSource::from_u8_vec(v))),
        scene_type: (SceneType, |v: &[u8]| Some(SceneType::from_u8_vec(v))),
        cfa_pattern: (CFAPattern, CFAPattern::from_vec),

        ambient_temperature: (AmbientTemperature, pick_v0_ir64),
        humidity: (Humidity, pick_v0_ur64),
        pressure: (Pressure, pick_v0_ur64),
        water_depth: (WaterDepth, pick_v0_ir64),
        acceleration: (Acceleration, pick_v0_ur64),
        camera_elevation_angle: (CameraElevationAngle, pick_v0_ir64),

        spectral_sensitivity: (SpectralSensitivity, some_string),
        // oecf: 未実装
        oecf: (OECF, |v: &[u8]| Some(v.to_vec())),
        subject_distance: (SubjectDistance, pick_v0_ur64),
        subject_distance_range: (SubjectDistanceRange, pick_v0),
        flash_energy: (FlashEnergy, pick_v0_ur64),
        // spatial_frequency_response: 未実装
        spatial_frequency_response: (SpatialFrequencyResponse, |v: &[u16]| Some(v.to_vec())),

        white_balance: (WhiteBalance, |v: &[u16]| Some(WhiteBalance::from_vec(v))),
        digital_zoom_ratio: (DigitalZoomRatio, pick_v0_ur64),
        focal_length_in_35mm_format: (FocalLengthIn35mmFormat, pick_v0),
        scene_capture_type: (SceneCaptureType, |v: &[u16]| Some(SceneCaptureType::from_vec(v))),
        gain_control: (GainControl, |v: &[u16]| Some(GainControl::from_vec(v))),
        contrast: (Contrast, |v: &[u16]| Some(Contrast::from_vec(v))),
        saturation: (Saturation, |v: &[u16]| Some(Saturation::from_vec(v))),
        sharpness: (Sharpness, |v: &[u16]| Some(Sharpness::from_vec(v))),
        custom_rendered: (CustomRendered, |v: &[u16]| Some(CustomRendered::from_vec(v))),
        // device_setting_description: 未実装
        device_setting_description: (DeviceSettingDescription, |v: &[u8]| Some(v.to_vec())),
        gamma: (Gamma, pick_v0_ur64),

        related_sound_file: (RelatedSoundFile, some_string),
        ;
    }
);

define_struct!(
    IdentifierInfo,
    fields: {
        image_unique_id: String,
    },
    tags: {
        image_unique_id: (ImageUniqueID, some_string),
        ;
    }
);

define_struct!(
    CompositeMetadata,
    fields: {
        composite_image: CompositeImage,
        composite_image_count: [u16; 2],
        composite_image_exposure_times: Vec<u8>,
    },
    tags: {
        composite_image: (CompositeImage, |v: &[u16]| Some(CompositeImage::from_vec(v))),
        composite_image_count: (CompositeImageCount, pick_v02),
        // composite_image_exposure_times: 未実装
        composite_image_exposure_times: (CompositeImageExposureTimes, |v: &[u8]| Some(v.to_vec()))
        ;
    }
);

pub fn parse_datetime(s: &str) -> Option<NaiveDateTime> {
    match NaiveDateTime::parse_from_str(s, "%Y:%m:%d %H:%M:%S") {
        Ok(ndt) => Some(ndt),
        Err(_) => None,
    }
}

pub fn load_components_configuration(v: &[u8]) -> Option<[ComponentsConfiguration; 4]> {
    if v.len() != 4 { return None; }
    let mut value = [
        ComponentsConfiguration::Unused,
        ComponentsConfiguration::Unused,
        ComponentsConfiguration::Unused,
        ComponentsConfiguration::Unused,
    ];
    for (i, &vi) in v.iter().enumerate() {
        value[i] = ComponentsConfiguration::from_vec(&[vi as u16]);
    }
    Some(value)
}

#[derive(Clone, PartialEq)]
pub struct TimeOffset {
    pub sign: bool,
    pub hour: u8,
    pub minute: u8,
}

impl TimeOffset {
    pub fn from_str(s: &str) -> Option<Self> {
        if s.len() != 6 { return None; }

        let sign = match &s[0..1] {
            "+" => true,
            "-" => false,
            _ => return None,
        };

        let hour: u8 = s[1..3].parse().ok()?;
        let minute: u8 = s[4..6].parse().ok()?;

        let mut none_flg = false;
        none_flg |= &s[3..4] != ":";
        none_flg |= minute != 0 && minute != 30;
        none_flg |= hour > 14;
        none_flg |= hour == 14 && minute == 30;
        none_flg |= hour >= 13 && !sign;

        if none_flg { return None; }

        Some(Self { sign, hour, minute })
    }
    pub fn to_string(&self) -> String {
        format!("{}{:>02}:{:>02}", if self.sign { "+" } else { "-" }, self.hour, self.minute)
    }
}

#[derive(Clone, PartialEq)]
pub struct CFAPattern {
    pub row: u16,
    pub column: u16,
    pub cfa: Vec<CFA>,
}

impl CFAPattern {
    pub fn from_vec(v: &[u8]) -> Option<Self> {
        let n = v.len();
        if n < 4 { return None; }
        let (row, column) = (u16::from_be_bytes([v[0], v[1]]), u16::from_be_bytes([v[2], v[3]]));
        if n != 4 + row as usize * column as usize { return None; }
        let cfa = v[4..].iter().map(|&vi| CFA::from_vec(&[vi as u16])).collect::<Vec<CFA>>();
        Some(Self { row, column, cfa })
    }

    pub fn to_vec(&self) -> Vec<u8> {
        let n = 4 + self.row as usize * self.column as usize;
        let mut v = vec![0; n];
        let row_bytes: [u8; 2] = self.row.to_be_bytes();
        v[0] = row_bytes[0];
        v[1] = row_bytes[1];
        let column_bytes: [u8; 2] = self.column.to_be_bytes();
        v[2] = column_bytes[0];
        v[3] = column_bytes[1];
        for i in 4..n { v[i] = self.cfa[i - 4].to_vec()[0] as u8; }
        v
    }
}

define_enum!(
    ExposureProgram {
        NotDefined = 0,
        Manual = 1,
        ProgramAE = 2,
        AperturePriorityAE = 3,
        ShutterSpeedPriorityAE = 4,
        CreativeSLowSpeed = 5,
        ActionHighSpeed = 6,
        Portrait = 7,
        Landscape = 8,
        Bulb = 9,
    }, 255
);

define_enum!(
    ExposureMode {
        Auto = 0,
        Manual = 1,
        AutoBracket = 2,
    }, 255
);

define_enum!(
    MeteringMode {
        Unknown = 0,
        Average = 1,
        CenterWeightedAverage = 2,
        Spot = 3,
        MultiSpot = 4,
        MultiSegment = 5,
        Partial = 6,
        Other = 255,
    }, 254
);

define_enum!(
    LightSource {
        Unknown = 0,
        Daylight = 1,
        Fluorescent = 2,
        TungstenIncandescent = 3,
        Flash = 4,
        FineWeather = 9,
        Cloudy = 10,
        Shade = 11,
        DaylightFluorescent = 12,
        DayWhiteFluorescent = 13,
        CoolWhiteFluorescent = 14,
        WhiteFluorescent = 15,
        WarmWhiteFluorescent = 16,
        StandardLightA = 17,
        StandardLightB = 18,
        StandardLightC = 19,
        D55 = 20,
        D65 = 21,
        D75 = 22,
        D50 = 23,
        ISOStudioTungsten = 24,
        Other = 255,
    }, 254
);

define_enum!(
    Flash {
        NoFlash = 0x0,
        Fired = 0x1,
        FiredReturnNotDetected = 0x5,
        FiredReturnDetected = 0x7,
        OnDidNotFire = 0x8,
        OnFired = 0x9,
        OnReturnNotDetected = 0xd,
        OnReturnDetected = 0xf,
        OffDidNotFire = 0x10,
        OffDidNotFireReturnNotDetected = 0x14,
        AutoDidNotFire = 0x18,
        AutoFired = 0x19,
        AutoFiredReturnNotDetected = 0x1d,
        AutoFiredReturnDetected = 0x1f,
        NoFlashFunction = 0x20,
        OffNoFlashFunction = 0x30,
        FiredRedEyeReduction = 0x41,
        FiredRedEyeReductionReturnNotDetected = 0x45,
        FiredRedEyeReductionReturnDetected = 0x47,
        OnRedEyeReduction = 0x49,
        OnRedEyeReductionReturnNotDetected = 0x4d,
        OnRedEyeReductionReturnDetected = 0x4f,
        OffRedEyeReduction = 0x50,
        AutoDidNotFireRedEyeReduction = 0x58,
        AutoFiredRedEyeReduction = 0x59,
        AutoFiredRedEyeReductionReturnNotDetected = 0x5d,
        AutoFiredRedEyeReductionReturnDetected = 0x5f,
    }, 0xff
);

define_enum!(
    SensitivityType {
        Unknown = 0,
        StandardOutputSensitivity = 1,
        RecommendedExposureIndex = 2,
        ISOSpeed = 3,
        StandardOutputSensitivityAndRecommendedExposureIndex = 4,
        StandardOutputSensitivityAndISOSpeed = 5,
        RecommendedExposureIndexAndISOSpeed = 6,
        StandardOutputSensitivityRecommendedExposureIndexAndISOSpeed = 7,
    }, 0xffff
);

define_enum!(
    ComponentsConfiguration {
        Unused = 0,
        Y = 1,
        Cb = 2,
        Cr = 3,
        R = 4,
        G = 5,
        B = 6,
    }, 0xffff
);

define_enum!(
    SensingMethod {
        MonochromeArea = 1,
        OneChipColorArea = 2,
        TwoChipColorArea = 3,
        ThreeChipColorArea = 4,
        ColorSequentialArea = 5,
        MonochromeLinear = 6,
        Trilinear = 7,
        ColorSequentialLinear = 8,
    }, 0
);

define_enum!(
    SceneType {
        DirectlyPhotographed = 1
    }, 0
);

define_enum!(
    CFA {
        Red = 0,
        Green = 1,
        Blue = 2,
        Cyan = 3,
        Magenta = 4,
        Yellow = 5,
        White = 6,
    }, 255
);

define_enum!(
    FocalPlaneResolutionUnit {
        None = 1,
        inches = 2,
        cm = 3,
        mm = 4,
        um = 5,
    }, 0
);

define_enum!(
    WhiteBalance {
        Auto = 0,
        Manual = 1,
    }, 255
);

define_enum!(
    SceneCaptureType {
        Standard = 0,
        Landscape = 1,
        Portrait = 2,
        Night = 3,
        Other = 4,
    }, 255
);

define_enum!(
    GainControl {
        None = 0,
        LowGainUp = 1,
        HighGainUp = 2,
        LowGainDown = 3,
        HighGainDown = 4,
    }, 255
);

define_enum!(
    Contrast {
        Normal = 0,
        Low = 1,
        High = 2,
    }, 255
);

define_enum!(
    Saturation {
        Normal = 0,
        Low = 1,
        High = 2,
    }, 255
);

define_enum!(
    Sharpness {
        Normal = 0,
        Soft = 1,
        Hard = 2,
    }, 255
);

define_enum!(
    CustomRendered {
        Normal = 0,
        Custom = 1,
        HDRNoOriginalSaved = 2,
        HDROriginalSaved = 3,
        OriginalForHDR = 4,
        Panorama = 6,
        PortraitHDR = 7,
        Portrait = 8
    }, 255
);

define_enum!(
    CompositeImage {
        Unknown = 0,
        NotAComPositeImage = 1,
        GeneralCompositeImage = 2,
        CompositeImageCapturesWhileShooting = 3
    }, 255
);

#[derive(Clone, PartialEq, Eq)]
pub enum FileSource {
    FilmScanner,
    ReflectionPrintScanner,
    DigitalCamera,
    SigmaDigitalCamera,
    UnknownValue(Vec<u16>),
}

impl FileSource {
    pub fn to_vec(&self) -> Vec<u16> {
        match self {
            Self::FilmScanner => vec![1],
            Self::ReflectionPrintScanner => vec![2],
            Self::DigitalCamera => vec![3],
            Self::SigmaDigitalCamera => vec![0x03, 0x00, 0x00, 0x00],
            Self::UnknownValue(v) => v.clone(),
        }
    }

    pub fn from_vec(v: &[u16]) -> Self {
        if v.len() != 1 {
            if v == vec![0x03, 0x00, 0x00, 0x00] {
                Self::SigmaDigitalCamera
            } else {
                Self::UnknownValue(v.to_vec())
            } 
        } else {
            match v[0] {
                1 => Self::FilmScanner,
                2 => Self::ReflectionPrintScanner,
                3 => Self::DigitalCamera,
                _ => Self::UnknownValue(v.to_vec()),
            }
        }
    }

    pub fn from_u8_vec(v: &[u8]) -> Self {
        Self::from_vec(&v.iter().map(|&vi| vi as u16).collect::<Vec<u16>>())
    }

    pub fn to_string(&self) -> String {
        match self {
            Self::FilmScanner => "FilmScanner".to_string(),
            Self::ReflectionPrintScanner => "ReflectionPrintScanner".to_string(),
            Self::DigitalCamera => "DigitalCamera".to_string(),
            Self::SigmaDigitalCamera => "SigmaDigitalCamera".to_string(),
            Self::UnknownValue(v) => format!("UnknownValue{:?}", v),
        }
    }

    

    pub fn unknown() -> Self { Self::UnknownValue(vec![0]) }
}

impl AllList for FileSource {
    fn to_u16(&self) -> u16 {
        self.to_vec()[0]
    }
    fn from_u16(value: u16) -> Self {
        Self::from_vec(&[value])
    }
    fn all(&self) -> Vec<(u16, Self)> {
        vec![
            (1, Self::FilmScanner),
            (2, Self::ReflectionPrintScanner),
            (3, Self::DigitalCamera),
            (4, Self::SigmaDigitalCamera),
            (5, Self::unknown())
        ]
    }
}

impl ShowValue for FileSource {
    fn show_value(&self) -> String { self.to_string() }
}

impl std::fmt::Debug for FileSource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_string())
    }
}

