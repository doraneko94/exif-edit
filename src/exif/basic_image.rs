use little_exif::exif_tag::ExifTag;
use little_exif::metadata::Metadata;

use super::utils::{
    F64, AllList, RemoveTag, 
    pick_v0, pick_v0_cast, pick_v0_ur64, 
    pick_v0768,
    pick_v02_ur64, pick_v03_ur64, pick_v04_ur64, pick_v06_ur64,
    some_string,
};

use crate::{define_big_struct, define_enum, define_struct};
use crate::components::utils::ShowValue;

define_big_struct!(
    BasicImageInfo,
    device_model: DeviceModel,
    image_format: ImageFormat,
    device_info: DeviceInfo,
);

define_struct!(
    DeviceModel,
    fields: {
        make: String,
        model: String,
        software: String,
    },
    tags: {
        make: (Make, some_string),
        model: (Model, some_string),
        software: (Software, some_string);
    }
);

define_struct!(
    ImageFormat,
    fields: {
        image_width: u32,
        image_height: u32,
        exif_image_width: u16,
        exif_image_height: u16,

        x_resolution: F64,
        y_resolution: F64,
        resolution_unit: ResolutionUnit,

        compression: Compression,
        photometric_interpretation: PhotometricInterpretation,
        color_space: ColorSpace,

        bits_per_sample: Vec<u16>,
        samples_per_pixel: u16,
        planar_configuration: PlanarConfiguration,
        ycbcr_sub_sampling: YCbCrSubSampling,
        ycbcr_positioning: YCbCrPositioning,
        ycbcr_coefficients: [F64; 3],

        transfer_function: [u16; 768],
        white_point: [F64; 2],
        primary_chromaticities: [F64; 6],
        reference_black_white: [F64; 6],
        color_map: Vec<u16>,

        strip_offsets: (Vec<u32>, Vec<Vec<u8>>),
        strip_byte_counts: Vec<u32>,
        rows_per_strip: u32,

        orientation: Orientation,
        cell_width: u16,
        cell_height: u16,
    },
    tags: {
        image_width: (ImageWidth, pick_v0),
        image_height: (ImageHeight, pick_v0),
        exif_image_width: (ExifImageWidth, pick_v0_cast),
        exif_image_height: (ExifImageHeight, pick_v0_cast),

        x_resolution: (XResolution, pick_v0_ur64),
        y_resolution: (YResolution, pick_v0_ur64),
        resolution_unit: (ResolutionUnit, |v: &[u16]| Some(ResolutionUnit::from_vec(v))),

        compression: (Compression, |v: &[u16]| Some(Compression::from_vec(v))),
        photometric_interpretation: (PhotometricInterpretation, |v: &[u16]| Some(PhotometricInterpretation::from_vec(v))),
        color_space: (ColorSpace, |v: &[u16]| Some(ColorSpace::from_vec(v))),

        bits_per_sample: (BitsPerSample, |v: &[u16]| Some(v.to_vec())),
        samples_per_pixel: (SamplesPerPixel, pick_v0),
        planar_configuration: (PlanarConfiguration, |v: &[u16]| Some(PlanarConfiguration::from_vec(v))),
        ycbcr_sub_sampling: (YCbCrSubSampling, |v: &[u16]| Some(YCbCrSubSampling::from_vec(v))),
        ycbcr_coefficients: (YCbCrCoefficients, pick_v03_ur64),

        transfer_function: (TransferFunction, pick_v0768),
        white_point: (WhitePoint, pick_v02_ur64),
        primary_chromaticities: (PrimaryChromaticities, pick_v06_ur64),
        reference_black_white: (ReferenceBlackWhite, pick_v06_ur64),
        color_map: (ColorMap, |v: &[u16]| Some(v.to_vec())),

        strip_byte_counts: (StripByteCounts, |v: &[u32]| Some(v.to_vec())),
        rows_per_strip: (RowsPerStrip, pick_v0),

        orientation: (Orientation, |v: &[u16]| Some(Orientation::from_vec(v))),
        cell_width: (CellWidth, pick_v0),
        cell_height: (CellHeight, pick_v0);

        strip_offsets: (StripOffsets, |v0: &[u32], v1: &[Vec<u8>]| Some((v0.to_vec(), v1.to_vec()))),
    }
    
);

define_struct!(
    DeviceInfo,
    fields: {
        serial_number: String,
        owner_name: String,
        lens_info: [F64; 4],
    },
    tags: {
        serial_number: (SerialNumber, some_string),
        owner_name: (OwnerName, some_string),
        lens_info: (LensInfo, pick_v04_ur64);
    }
);

define_enum!(
    ResolutionUnit {
        None = 1,
        Inches = 2,
        cm = 3,
    }, 0
);

define_enum!(
    Compression {
        Uncompressed = 1,
        CCITT1D = 2,
        T4Group3Fax = 3,
        T6Group4Fax = 4,
        LZW = 5,
        JPEGOldStyle = 6,
        JPEG = 7,
        AdobeDeflate = 8,
        JBIGBW = 9,
        JBIGColor = 10,
        JPEG_ = 99,
        Kodak262 = 262,
        Next = 32766,
        SonyARWCompressed = 32767,
        PackedRAW = 32769,
        SamsungSRWCompressed = 32770,
        CCIRLEW = 32771,
        SamsungSRWCompressed2 = 32772,
        PackBits = 32773,
        Thunderscan = 32809,
        KodakKDCCompressed = 32867,
        IT8CTPAD = 32895,
        IT8LW = 32896,
        IT8MP = 32897,
        IT8BL = 32898,
        PixarFilm = 32908,
        PixarLog = 32909,
        Deflate = 32946,
        DCS = 32947,
        AperioJPEG2000YCbCr = 33003,
        AperioJPEG2000RGB = 33005,
        JBIG = 34661,
        SGILog = 34676,
        SGILog24 = 34677,
        JPEG2000 = 34712,
        NikonNEFCompressed = 34713,
        JBIG2TIFFFX = 34715,
        MicrosoftDocumentImagingMDIBinaryLevelCodec = 34718,
        MicrosoftDocumentImagingMDIProgressiveTransformCodec = 34719,
        MicrosoftDocumentImagingMDIVector = 34720,
        ESRILerc = 34887,
        LossyJPEG = 34892,
        LZMA2 = 34925,
        ZstdOld = 34926,
        WebPOld = 34927,
        PNG = 34933,
        JPEGXR = 34934,
        Zstd = 50000,
        WebP = 50001,
        JPEGXLOld = 50002,
        JPEGXL = 52546,
        KodakDCRCompressed = 65000,
        PentaxPEFCompressed = 65535,
    }, 0
);

define_enum!(
    PhotometricInterpretation {
        WhiteIsZero = 0,
        BlackIsZero = 1,
        RGB = 2,
        RGBPalette = 3,
        TransparencyMask = 4,
        CMYK = 5,
        YCbCr = 6,
        CIELab = 8,
        ICCLab = 9,
        ITULab = 10,
        ColorFilterArray = 32803,
        PixarLogL = 32844,
        PixarLogLuv = 32845,
        SequentialColorFilter = 32892,
        LinearRaw = 34892,
        DepthMap = 51177,
        SemanticMask = 52527,
    }, 65535
);

define_enum!(
    ColorSpace {
        sRGB = 0x1,
        AdobeRGB = 0x2,
        WideGamutRGB = 0xfffd,
        ICCProfile = 0xfffe,
        UnCalibrated = 0xffff
    }, 0
);

define_enum!(
    PlanarConfiguration {
        Chunky = 1,
        Planar = 2,
    }, 0
);

#[derive(Clone, PartialEq)]
#[allow(non_camel_case_types)]
pub enum YCbCrSubSampling {
    YCbCr444_11,
    YCbCr440_12,
    YCbCr441_14,
    YCbCr422_21,
    YCbCr420_22,
    YCbCr421_24,
    YCbCr411_41,
    YCbCr410_42,
    Unknown(Vec<u16>)
}

impl YCbCrSubSampling {
    pub fn to_vec(&self) -> Vec<u16> {
        match self {
            Self::YCbCr444_11 => vec![1, 1],
            Self::YCbCr440_12 => vec![1, 2],
            Self::YCbCr441_14 => vec![1, 4],
            Self::YCbCr422_21 => vec![2, 1],
            Self::YCbCr420_22 => vec![2, 2],
            Self::YCbCr421_24 => vec![2, 4],
            Self::YCbCr411_41 => vec![4, 1],
            Self::YCbCr410_42 => vec![4, 2],
            Self::Unknown(v) => v.clone()
        }
    }

    pub fn from_vec(v: &[u16]) -> Self {
        if v.len() != 2 {
            Self::Unknown(v.to_vec())
        } else {
            match v {
                &[1, 1] => Self::YCbCr444_11,
                &[1, 2] => Self::YCbCr440_12,
                &[1, 4] => Self::YCbCr441_14,
                &[2, 1] => Self::YCbCr422_21,
                &[2, 2] => Self::YCbCr420_22,
                &[2, 4] => Self::YCbCr421_24,
                &[4, 1] => Self::YCbCr411_41,
                &[4, 2] => Self::YCbCr410_42,
                _ => Self::Unknown(v.to_vec())
            }
        }
    }

    pub fn to_string(&self) -> String {
        match self {
            Self::YCbCr444_11 => "YCbCr4:4:4 (1 1)".to_string(),
            Self::YCbCr440_12 => "YCbCr4:4:0 (1 2)".to_string(),
            Self::YCbCr441_14 => "YCbCr4:4:1 (1 4)".to_string(),
            Self::YCbCr422_21 => "YCbCr4:2:2 (2 1)".to_string(),
            Self::YCbCr420_22 => "YCbCr4:2:0 (2 2)".to_string(),
            Self::YCbCr421_24 => "YCbCr4:2:1 (2 4)".to_string(),
            Self::YCbCr411_41 => "YCbCr4:1:1 (4 1)".to_string(),
            Self::YCbCr410_42 => "YCbCr4:1:0 (4 2)".to_string(),
            Self::Unknown(v) => format!("Unknown({:?})", v),
        }
    }

    pub fn unknown() -> Self { Self::Unknown(vec![0, 0]) }
}

impl AllList for YCbCrSubSampling {
    fn to_u16(&self) -> u16 {
        match self {
            Self::YCbCr444_11 => 0,
            Self::YCbCr440_12 => 1,
            Self::YCbCr441_14 => 2,
            Self::YCbCr422_21 => 3,
            Self::YCbCr420_22 => 4,
            Self::YCbCr421_24 => 5,
            Self::YCbCr411_41 => 6,
            Self::YCbCr410_42 => 7,
            Self::Unknown(_) => 8,
        }
    }
    fn from_u16(value: u16) -> Self {
        match value {
            0 => Self::YCbCr444_11,
            1 => Self::YCbCr440_12,
            2 => Self::YCbCr441_14,
            3 => Self::YCbCr422_21,
            4 => Self::YCbCr420_22,
            5 => Self::YCbCr421_24,
            6 => Self::YCbCr411_41,
            7 => Self::YCbCr410_42,
            val => Self::Unknown(vec![val]),
        }
    }
    fn all(&self) -> Vec<(u16, Self)> {
        let mut v = vec![
            (0, Self::YCbCr444_11),
            (1, Self::YCbCr440_12),
            (2, Self::YCbCr441_14),
            (3, Self::YCbCr422_21),
            (4, Self::YCbCr420_22),
            (5, Self::YCbCr421_24),
            (6, Self::YCbCr411_41),
            (7, Self::YCbCr410_42)
        ];
        match self {
            Self::Unknown(value) => { v.push((8, Self::Unknown(value.clone()))) }
            _ => {}
        }
        v
    }
}

impl ShowValue for YCbCrSubSampling {
    fn show_value(&self) -> String { self.to_string() }
}

impl std::fmt::Debug for YCbCrSubSampling {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_string())
    }
}

define_enum!(
    YCbCrPositioning {
        Centered = 1,
        CoSited = 2,
    }, 0
);

define_enum!(
    Orientation {
        Horizontal = 1,
        MirrorHorizontal = 2,
        Rotate180 = 3,
        MirrorVertical = 4,
        MirrorHorizontalAndRotate270CW = 5,
        Rotate90CW = 6,
        MirrorHorizontalAndRotate90CW = 7,
        Rotate270CW = 8,
    }, 0
);