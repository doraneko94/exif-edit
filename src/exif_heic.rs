use little_exif::exif_tag::ExifTag;
use little_exif::metadata::Metadata;
use little_exif::rational::{iR64, uR64};
use num_traits::NumCast;
use serde_json::Value;

use super::exif::rational::approx_frac;

macro_rules! match_string {
    ($tag:ident, $value:ident, $metadata:ident) => {
        match $value {
            Value::String(s) => $metadata.set_tag(ExifTag::$tag(s.to_string())),
            Value::Array(a) => $metadata.set_tag(ExifTag::$tag(
                a.iter().map(|ai| match ai {
                    Value::String(s) => s.to_string(),
                    _ => "".to_string()
                }).collect::<Vec<String>>().join(", ")
            )),
            _ => {}
        }
    };
}

macro_rules! match_vec_number {
    ($tag:ident, $value:ident, $metadata:ident) => {
        match $value {
            Value::Number(n) => if let Some(n64) = n.as_u64() {
                if let Some(value) = NumCast::from(n64) { $metadata.set_tag(ExifTag::$tag(vec![value])); }
            },
            Value::Array(a) => {
                let mut v = Vec::with_capacity(a.len());
                let mut all_some = true;
                for ai in a.iter() {
                    if let Value::Number(n) = ai {
                        match n.as_u64() {
                            Some(n64) => match NumCast::from(n64) {
                                Some(value) => { v.push(value); }
                                None => { all_some = false; break; }
                            }
                            None => { all_some = false; break; }
                        }
                    } else { all_some = false; break; }
                }
                if all_some { $metadata.set_tag(ExifTag::$tag(v)); }
            }
            _ => {}
        }
    }
}

macro_rules! match_vec_rational {
    ($rational:ident, $tag:ident, $value:ident, $metadata:ident) => {
        match $value {
            Value::Number(n) => if let Some(f) = n.as_f64() {
                if let Some((_, nom, den)) = approx_frac(f) {
                    if let (Some(nominator), Some(denominator)) = (NumCast::from(nom), NumCast::from(den)) {
                        $metadata.set_tag(ExifTag::$tag(vec![$rational { nominator, denominator }]));
                    }
                }
            }
            Value::Array(a) => {
                let mut v = Vec::with_capacity(a.len());
                let mut all_some = true;
                for ai in a.iter() {
                    if let Value::Number(n) = ai {
                        match n.as_f64() {
                            Some(f) => match approx_frac(f) {
                                Some((_, nom, den)) => {
                                    match (NumCast::from(nom), NumCast::from(den)) {
                                        (Some(nominator), Some(denominator)) => {
                                            v.push($rational { nominator, denominator });
                                        }
                                        _ => { all_some = false; break; }
                                    }
                                }
                                None => { all_some = false; break; }
                            }
                            None => { all_some = false; break; }
                        }
                    } else { all_some = false; break; }
                }
                if all_some { $metadata.set_tag(ExifTag::$tag(v)); }
            }
            _ => {}
        }
    };
}

pub fn metadata_heic(exif_dict: &Value) -> Metadata {
    let mut metadata = Metadata::new();
    if let Value::Object(map) = exif_dict {
        for (key, value) in map.iter() {
            match key.as_str() {
                "Make" => match_string!(Make, value, metadata),
                "Model" => match_string!(Model, value, metadata),
                "Software" => match_string!(Software, value, metadata),

                "ImageWidth" => match_vec_number!(ImageWidth, value, metadata),
                "ImageHeight" => match_vec_number!(ImageHeight, value, metadata),
                "ExifImageWidth" => match_vec_number!(ExifImageWidth, value, metadata),
                "ExifImageHeight" => match_vec_number!(ExifImageHeight, value, metadata),

                "XResolution" => match_vec_rational!(uR64, XResolution, value, metadata),
                "YResolution" => match_vec_rational!(uR64, YResolution, value, metadata),
                "ResolutionUnit" => match_vec_number!(ResolutionUnit, value, metadata),

                "Compression" => match_vec_number!(Compression, value, metadata),
                "PhotometricInterpretation" => match_vec_number!(PhotometricInterpretation, value, metadata),
                "ColorSpace" => match_vec_number!(ColorSpace, value, metadata),

                "BitsPerSample" => match_vec_number!(BitsPerSample, value, metadata),
                "SamplesPerPixel" => match_vec_number!(SamplesPerPixel, value, metadata),
                "PlanarConfiguration" => match_vec_number!(PlanarConfiguration, value, metadata),
                "YCbCrSubSampling" => match_vec_number!(YCbCrSubSampling, value, metadata),
                "YCbCrPositioning" => match_vec_number!(YCbCrPositioning, value, metadata),
                "YCbCrCoefficients" => match_vec_rational!(uR64, YCbCrCoefficients, value, metadata),

                "TransferFunction" => match_vec_number!(TransferFunction, value, metadata),
                "WhitePoint" => match_vec_rational!(uR64, WhitePoint, value, metadata),
                "PrimaryChromaticities" => match_vec_rational!(uR64, PrimaryChromaticities, value, metadata),
                "ReferenceBlackWhite" => match_vec_rational!(uR64, ReferenceBlackWhite, value, metadata),
                "ColorMap" => match_vec_number!(ColorMap, value, metadata),

                // "StripOffsets" =>
                "StripByteCounts" => match_vec_number!(StripByteCounts, value, metadata),
                "RowsPerStrip" => match_vec_number!(RowsPerStrip, value, metadata),
                
                "Orientation" => match_vec_number!(Orientation, value, metadata),
                "CellWidth" => match_vec_number!(CellWidth, value, metadata),
                "CellLength" => match_vec_number!(CellHeight, value, metadata),

                "SerialNumber" => match_string!(SerialNumber, value, metadata),
                "OwnerName" => match_string!(OwnerName, value, metadata),
                "LensInfo" => match_vec_rational!(uR64, LensInfo, value, metadata),

                "DateTimeOriginal" => match_string!(DateTimeOriginal, value, metadata),
                "OffsetTimeOriginal" => match_string!(OffsetTimeOriginal, value, metadata),
                "SubSecTimeOriginal" => match_string!(SubSecTimeOriginal, value, metadata),

                "CreateDate" => match_string!(CreateDate, value, metadata),
                "OffsetTimeDigitized" => match_string!(OffsetTimeDigitized, value, metadata),
                "SubSecTimeDigitized" => match_string!(SubSecTimeDigitized, value, metadata),

                "ModifyDate" => match_string!(ModifyDate, value, metadata),
                "OffsetTime" => match_string!(OffsetTime, value, metadata),
                "SubSecTime" => match_string!(SubSecTime, value, metadata),
                
                "LensMake" => match_string!(LensMake, value, metadata),
                "LensModel" => match_string!(LensModel, value, metadata),
                "LensSerialNumber" => match_string!(LensSerialNumber, value, metadata),
                "MaxApertureValue" => match_vec_rational!(uR64, MaxApertureValue, value, metadata),

                "ExposureProgram" => match_vec_number!(ExposureProgram, value, metadata),
                "ExposureMode" => match_vec_number!(ExposureMode, value, metadata),

                "ExposureTime" => match_vec_rational!(uR64, ExposureTime, value, metadata),
                "ShutterSpeedValue" => match_vec_rational!(iR64, ShutterSpeedValue, value, metadata),

                "FNumber" => match_vec_rational!(uR64, FNumber, value, metadata),
                "ApertureValue" => match_vec_rational!(uR64, ApertureValue, value, metadata),

                "ExposureCompensation" => match_vec_rational!(iR64, ExposureCompensation, value, metadata),
                "BrightnessValue" => match_vec_rational!(iR64, BrightnessValue, value, metadata),

                "MeteringMode" => match_vec_number!(MeteringMode, value, metadata),
                "LightSource" => match_vec_number!(LightSource, value, metadata),
                "Flash" => match_vec_number!(Flash, value, metadata),

                "FocalLength" => match_vec_rational!(uR64, FocalLength, value, metadata),
                "SubjectArea" => match_vec_number!(SubjectArea, value, metadata),
                "SubjectLocation" => match_vec_number!(SubjectLocation, value, metadata),

                "SensitivityType" => match_vec_number!(SensitivityType, value, metadata),

                "ISO" => match_vec_number!(ISO, value, metadata),
                "ISOSpeed" => match_vec_number!(ISOSpeed, value, metadata),

                "StandardOutputSensitivity" => match_vec_number!(StandardOutputSensitivity, value, metadata),
                "RecommendedExposureIndex" => match_vec_number!(RecommendedExposureIndex, value, metadata),
                "ExposureIndex" => match_vec_rational!(uR64, ExposureIndex, value, metadata),

                "ISOSpeedLatitudeyyy" => match_vec_number!(ISOSpeedLatitudeyyy, value, metadata),
                "ISOSpeedLatitudezzz" => match_vec_number!(ISOSpeedLatitudezzz, value, metadata),

                "ExifVersion" => match_vec_number!(ExifVersion, value, metadata),
                "FlashpixVersion" => match_vec_number!(FlashpixVersion, value, metadata),
                "ExifOffset" => match_vec_number!(ExifOffset, value, metadata),
                "ComponentsConfiguration" => match_vec_number!(ComponentsConfiguration, value, metadata),
                "CompressedBitsPerPixel" => match_vec_rational!(uR64, CompressedBitsPerPixel, value, metadata),

                "SensingMethod" => match_vec_number!(SensingMethod, value, metadata),
                "FileSource" => match_vec_number!(FileSource, value, metadata),
                "SceneType" => match_vec_number!(SceneType, value, metadata),
                "CFAPattern" => match_vec_number!(CFAPattern, value, metadata),

                "AmbientTemperature" => match_vec_rational!(iR64, AmbientTemperature, value, metadata),
                "Humidity" => match_vec_rational!(uR64, Humidity, value, metadata),
                "Pressure" => match_vec_rational!(uR64, Pressure, value, metadata),
                "WaterDepth" => match_vec_rational!(iR64, WaterDepth, value, metadata),
                "Acceleration" => match_vec_rational!(uR64, Acceleration, value, metadata),
                "CameraElevationAngle" => match_vec_rational!(iR64, CameraElevationAngle, value, metadata),

                "SpectralSensitivity" => match_string!(SpectralSensitivity, value, metadata),
                "Opto-ElectricConvFactor" => match_vec_number!(OECF, value, metadata),
                "SubjectDistance" => match_vec_rational!(uR64, SubjectDistance, value, metadata),
                "SubjectDistanceRange" => match_vec_number!(SubjectDistanceRange, value, metadata),
                "FlashEnergy" => match_vec_rational!(uR64, FlashEnergy, value, metadata),
                "SpatialFrequencyResponse" => match_vec_number!(SpatialFrequencyResponse, value, metadata),

                "FocalPlaneXResolution" => match_vec_rational!(uR64, FocalPlaneXResolution, value, metadata),
                "FocalPlaneYResolution" => match_vec_rational!(uR64, FocalPlaneYResolution, value, metadata),
                "FocalPlaneResolutionUnit" => match_vec_number!(FocalPlaneResolutionUnit, value, metadata),

                "WhiteBalance" => match_vec_number!(WhiteBalance, value, metadata),
                "DigitalZoomRatio" => match_vec_rational!(uR64, DigitalZoomRatio, value, metadata),
                "FocalLengthIn35mmFormat" => match_vec_number!(FocalLengthIn35mmFormat, value, metadata),
                "SceneCaptureType" => match_vec_number!(SceneCaptureType, value, metadata),
                "GainControl" => match_vec_number!(GainControl, value, metadata),
                "Contrast" => match_vec_number!(Contrast, value, metadata),
                "Saturation" => match_vec_number!(Saturation, value, metadata),
                "Sharpness" => match_vec_number!(Sharpness, value, metadata),
                "CustomRendered" => match_vec_number!(CustomRendered, value, metadata),
                "DeviceSettingDescription" => match_vec_number!(DeviceSettingDescription, value, metadata),
                "Gamma" => match_vec_rational!(uR64, Gamma, value, metadata),

                "RelatedSoundFile" => match_string!(RelatedSoundFile, value, metadata),
                "ImageUniqueID" => match_string!(ImageUniqueID, value, metadata),

                "CompositeImage" => match_vec_number!(CompositeImage, value, metadata),
                "CompositeImageCount" => match_vec_number!(CompositeImageCount, value, metadata),
                "CompositeImageExposureTimes" => match_vec_number!(CompositeImageExposureTimes, value, metadata),

                "GPSLatitudeRef" => match_string!(GPSLatitudeRef, value, metadata),
                "GPSLatitude" => match_vec_rational!(uR64, GPSLatitude, value, metadata),
                "GPSLongitudeRef" => match_string!(GPSLongitudeRef, value, metadata),
                "GPSLongitude" => match_vec_rational!(uR64, GPSLongitude, value, metadata),
                "GPSMapDatum" => match_string!(GPSMapDatum, value, metadata),
                "GPSAltitudeRef" => match_vec_number!(GPSAltitudeRef, value, metadata),
                "GPSAltitude" => match_vec_rational!(uR64, GPSAltitude, value, metadata),

                "GPSSatellites" => match_string!(GPSSatellites, value, metadata),
                "GPSStatus" => match_string!(GPSStatus, value, metadata),
                "GPSMeasureMode" => match_string!(GPSMeasureMode, value, metadata),
                "GPSDOP" => match_vec_rational!(uR64, GPSDOP, value, metadata),

                "GPSSpeedRef" => match_string!(GPSSpeedRef, value, metadata),
                "GPSSpeed" => match_vec_rational!(uR64, GPSSpeed, value, metadata),
                "GPSTrackRef" => match_string!(GPSTrackRef, value, metadata),
                "GPSTrack" => match_vec_rational!(uR64, GPSTrack, value, metadata),
                "GPSImgDirectionRef" => match_string!(GPSImgDirectionRef, value, metadata),
                "GPSImgDirection" => match_vec_rational!(uR64, GPSImgDirection, value, metadata),

                "GPSDestLatitudeRef" => match_string!(GPSDestLatitudeRef, value, metadata),
                "GPSDestLatitude" => match_vec_rational!(uR64, GPSDestLatitude, value, metadata),
                "GPSDestLongitudeRef" => match_string!(GPSDestLongitudeRef, value, metadata),
                "GPSDestLongitude" => match_vec_rational!(uR64, GPSDestLongitude, value, metadata),
                "GPSDestBearingRef" => match_string!(GPSDestBearingRef, value, metadata),
                "GPSDestBearing" => match_vec_rational!(uR64, GPSDestBearing, value, metadata),
                "GPSDestDistanceRef" => match_string!(GPSDestDistanceRef, value, metadata),
                "GPSDestDistance" => match_vec_rational!(uR64, GPSDestDistance, value, metadata),

                "GPSProcessingMethod" => match_vec_number!(GPSProcessingMethod, value, metadata),
                "GPSAreaInformation" => match_vec_number!(GPSAreaInformation, value, metadata),
                "GPSDateStamp" => match_string!(GPSDateStamp, value, metadata),
                "GPSTimeStamp" => match_vec_rational!(uR64, GPSTimeStamp, value, metadata),
                "GPSDifferential" => match_vec_number!(GPSDifferential, value, metadata),
                "GPSHPositioningError" => match_vec_rational!(uR64, GPSHPositioningError, value, metadata),
                "GPSVersionID" => match_vec_number!(GPSVersionID, value, metadata),
                "GPSInfo" => match_vec_number!(GPSInfo, value, metadata),

                "InteropOffset" => match_vec_number!(InteropOffset, value, metadata),
                "InteropIndex" => match_string!(InteroperabilityIndex, value, metadata),
                "InteropVersion" => match_vec_number!(InteroperabilityVersion, value, metadata),

                // "ThumbnailOffset" => 
                "ThumbnailLength" => match_vec_number!(ThumbnailLength, value, metadata),

                "ImageDescription" => match_string!(ImageDescription, value, metadata),
                "Artist" => match_string!(Artist, value, metadata),
                "Copyright" => match_string!(Copyright, value, metadata),
                "UserComment" => match_vec_number!(UserComment, value, metadata),

                "MakerNote" => match_vec_number!(MakerNote, value, metadata),

                _ => {}
            }
        }
    }
    metadata
}