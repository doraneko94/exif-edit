use std::fmt;

use chrono::Local;
use little_exif::ifd::ExifTagGroup;
use little_exif::metadata::Metadata;
use little_exif::exif_tag::ExifTag;
use little_exif::rational::uR64;

pub mod basic_image;
pub mod exif_capture;
pub mod gps;
pub mod interop;
pub mod thumbnail;
pub mod user;
pub mod uneditable;
pub mod rational;
pub mod utils;

use basic_image::{
    BasicImageInfo,
    ResolutionUnit, Compression, PhotometricInterpretation, ColorSpace,
    PlanarConfiguration, YCbCrSubSampling, YCbCrPositioning, Orientation
};
use exif_capture::{
    ExifCaptureInfo,
    TimeOffset, ExposureProgram, ExposureMode,
    MeteringMode, LightSource, Flash, SensitivityType,
    load_components_configuration, parse_datetime,
    SensingMethod, FileSource, SceneType, CFAPattern,
    FocalPlaneResolutionUnit, WhiteBalance, SceneCaptureType,
    GainControl, Contrast, Saturation, Sharpness, CustomRendered,
    CompositeImage
};
use gps::{
    GpsInfo, DMS, 
    GPSAltitudeRef, GPSStatus, GPSMeasureMode, GPSSpeedRef,
    NorthRef, GPSDestDistanceRef, GPSDifferential,
    gps_ref, parse_date, parse_time
};
use interop::{InteropInfo, InteroperabilityIndex};
use thumbnail::ThumbnailInfo;
use user::{UserInfo, UserComment};
use uneditable::Uneditable;
use rational::ExifRational;
use utils::{
    VersionAscii,
    pick_v0, pick_v0_cast, pick_v0_ur64, pick_v0_ir64,
    pick_v02, pick_v04, pick_v0768,
    pick_v02_ur64, pick_v03_ur64, pick_v04_ur64, pick_v06_ur64,
};

use crate::components::utils::ShowValue;

#[derive(Clone)]
pub enum ExifTime {
    DateTimeOriginal,
    CreateDate,
    ModifyDate
}

#[derive(Clone, Debug)]
pub struct Coordinate {
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

#[derive(Clone, PartialEq)]
pub struct ExifEditData {
    pub metadata: Metadata,

    pub basic_image_info: BasicImageInfo,
    pub exif_capture_info: ExifCaptureInfo,
    pub gps_info: GpsInfo,
    pub interop_info: InteropInfo,
    pub thumbnail_info: ThumbnailInfo,
    pub user_info: UserInfo,

    pub uneditable: Uneditable,
}

impl ExifEditData {
    pub fn empty() -> Self {
        Self {
            metadata: Metadata::new(),

            basic_image_info: BasicImageInfo::new(),
            exif_capture_info: ExifCaptureInfo::new(),
            gps_info: GpsInfo::new(),
            interop_info: InteropInfo::new(),
            thumbnail_info: ThumbnailInfo::new(),
            user_info: UserInfo::new(),

            uneditable: Uneditable::new(),
        }
    }

    pub fn new(metadata: &Metadata) -> Self {
        let mut ret = Self::empty();
        
        for ifd in metadata.get_ifds() {
            for tag in ifd.get_tags() {
                ret.update_tag(tag.clone());
            }
        }
        ret
    }

    pub fn update_tag(&mut self, tag: ExifTag) {
        match &tag {
            // BasicImageInfo（基本画像情報）（ここから）
            // DeviceModel
            // カメラ本体の製造元
            ExifTag::Make(s) => { self.basic_image_info.device_model.make = Some(s.clone()); }
            // カメラ本体のモデル名
            ExifTag::Model(s) => { self.basic_image_info.device_model.model = Some(s.clone()); }
            // 撮影画像に使用されたソフトウェア（ファームウェアや編集ソフト）
            ExifTag::Software(s) => { self.basic_image_info.device_model.software = Some(s.clone()); }

            // 0th ImageFormat
            // 現在の画像の見た目のサイズ
            // 画像サイズ情報（論理画素数）：ImageWidth/ExifImageWidthとして表示
            ExifTag::ImageWidth(v) => { self.basic_image_info.image_format.image_width = pick_v0(v); }
            // （更新時にStripOffsets, StripByteCountsの長さ修正）
            ExifTag::ImageHeight(v) => { self.basic_image_info.image_format.image_height = pick_v0(v); }
            // 撮影時のオリジナルの画像サイズ
            ExifTag::ExifImageWidth(v) => { self.basic_image_info.image_format.exif_image_width = pick_v0_cast(v); }
            ExifTag::ExifImageHeight(v) => { self.basic_image_info.image_format.exif_image_height = pick_v0_cast(v); }

            // 解像度情報
            ExifTag::XResolution(v) => { self.basic_image_info.image_format.x_resolution = pick_v0_ur64(v); }
            ExifTag::YResolution(v) => { self.basic_image_info.image_format.y_resolution = pick_v0_ur64(v); }
            ExifTag::ResolutionUnit(v) => { self.basic_image_info.image_format.resolution_unit = Some(ResolutionUnit::from_vec(v)); }

            // 圧縮・色空間・符号か情報（主要フォーマット定義）
            // データ形式
            ExifTag::Compression(v) => { self.basic_image_info.image_format.compression = Some(Compression::from_vec(v)); }
            // カラーモデル
            ExifTag::PhotometricInterpretation(v) => { self.basic_image_info.image_format.photometric_interpretation = Some(PhotometricInterpretation::from_vec(v)); }
            // 色空間
            ExifTag::ColorSpace(v) => { self.basic_image_info.image_format.color_space = Some(ColorSpace::from_vec(v)); },

            // 色成分関連（ビット構成、サンプリング）
            // 各色チャネルあたりのビット深度：SamplesPerPixelの長さに相当（[0]更新時にColorMapの長さ修正）
            ExifTag::BitsPerSample(v) => { self.basic_image_info.image_format.bits_per_sample = Some(v.clone()); }
            // ピクセルあたりのチャネル数：更新時にBitsPerSamplesの長さ修正
            ExifTag::SamplesPerPixel(v) => { self.basic_image_info.image_format.samples_per_pixel = pick_v0(v); }
            // チャネルの配置形式
            ExifTag::PlanarConfiguration(v) => { self.basic_image_info.image_format.planar_configuration = Some(PlanarConfiguration::from_vec(v)); }
            // サブサンプリング方式
            ExifTag::YCbCrSubSampling(v) => { self.basic_image_info.image_format.ycbcr_sub_sampling = Some(YCbCrSubSampling::from_vec(v)); }
            // サブサンプリングされた成分の位置づけ
            ExifTag::YCbCrPositioning(v) => { self.basic_image_info.image_format.ycbcr_positioning = Some(YCbCrPositioning::from_vec(v)); }
            // RGB→YCbCr変換の係数：PhotometricInterpretation=6のときに意味をもつ3次元ベクトル [Kr, Kg, Kb]
            ExifTag::YCbCrCoefficients(v) => { self.basic_image_info.image_format.ycbcr_coefficients = pick_v03_ur64(v); }

            // トーンカーブと色変換
            // トーン再現カーブ（LUT）：長さ768
            ExifTag::TransferFunction(v) => { self.basic_image_info.image_format.transfer_function = pick_v0768(v); }
            // 色の基準点（白のCIE座標）：長さ2
            ExifTag::WhitePoint(v) => { self.basic_image_info.image_format.white_point = pick_v02_ur64(v); }
            // RGBそれぞれの原色の色度点：長さ6
            ExifTag::PrimaryChromaticities(v) => { self.basic_image_info.image_format.primary_chromaticities = pick_v06_ur64(v); }
            // 各チャネルの黒・白の基準値：長さ6
            ExifTag::ReferenceBlackWhite(v) => { self.basic_image_info.image_format.reference_black_white = pick_v06_ur64(v); }
            // インデックスカラーモード用のカラー定義：PhotometricInterpretation=6のときに意味をもつ 3 * 2^{BitsPerSample[0]} 次元ベクトル
            ExifTag::ColorMap(v) => { self.basic_image_info.image_format.color_map = Some(v.clone()); }

            // ストリップ情報（画像データ配置）
            // 画像データの格納開始位置：配列の長さはceil(ImageHeight / RowsPerStrip)
            ExifTag::StripOffsets(v0, v1) => {
                self.basic_image_info.image_format.strip_offsets = Some((v0.clone(), v1.clone()));
            }
            // 各ストリップのデータ量：配列の長さはceil(ImageHeight / RowsPerStrip)
            ExifTag::StripByteCounts(v) => { self.basic_image_info.image_format.strip_byte_counts = Some(v.clone()); }
            // ストリップ単位あたりの行数（画像の分割単位）：スカラ（更新時にStripOffsets, StripByteCountsの長さ修正）
            ExifTag::RowsPerStrip(v) => { self.basic_image_info.image_format.rows_per_strip = pick_v0(v); }

            // 補助構造・その他
            // 回転・反転情報
            ExifTag::Orientation(v) => { self.basic_image_info.image_format.orientation = Some(Orientation::from_vec(v)); }
            // 古いTIFF形式でのセルの大きさ（非推奨）
            ExifTag::CellWidth(v) => { self.basic_image_info.image_format.cell_width = pick_v0(v); }
            ExifTag::CellHeight(v) => { self.basic_image_info.image_format.cell_height = pick_v0(v); }
            
            // DeviceInfo
            // カメラ本体の固有ID
            ExifTag::SerialNumber(s) => { self.basic_image_info.device_info.serial_number = Some(s.clone()); }
            // カメラ本体の所有者
            ExifTag::OwnerName(s) => { self.basic_image_info.device_info.owner_name = Some(s.clone()); }
            // 装着レンズの仕様：長さ4
            ExifTag::LensInfo(v) => { self.basic_image_info.device_info.lens_info = pick_v04_ur64(v); }
            // BasicImageInfo（基本画像情報）（ここまで）

            // ExifCaptureInfo（撮影関連情報）（ここから）
            // TimeInfo
            // 画像の撮影日時
            ExifTag::DateTimeOriginal(s) => if let Some(ndt) = parse_datetime(s) {
                self.exif_capture_info.time_info.date_time_original = Some(ndt);
                if self.exif_capture_info.time_info.sub_sec_time_original.is_none() {
                    self.exif_capture_info.time_info.sub_sec_time_original = Some(0);
                }
            }
            // タイムゾーン情報
            ExifTag::OffsetTimeOriginal(s) => { self.exif_capture_info.time_info.offset_time_original = TimeOffset::from_str(s); }
            // ミリ秒以下
            ExifTag::SubSecTimeOriginal(s) => if let Ok(u) = s.parse() { 
                self.exif_capture_info.time_info.sub_sec_time_original = Some(u);
            }

            // 画像のデジタル化（ファイルに保存）された日時
            ExifTag::CreateDate(s) => if let Some(ndt) = parse_datetime(s) {
                self.exif_capture_info.time_info.create_date = Some(ndt);
                if self.exif_capture_info.time_info.sub_sec_time_digitized.is_none() {
                    self.exif_capture_info.time_info.sub_sec_time_digitized = Some(0);
                }
            }
            // タイムゾーン情報
            ExifTag::OffsetTimeDigitized(s) => { self.exif_capture_info.time_info.offset_time_digitized = TimeOffset::from_str(s); }
            // ミリ秒以下
            ExifTag::SubSecTimeDigitized(s) => if let Ok(u) = s.parse() {
                self.exif_capture_info.time_info.sub_sec_time_digitized = Some(u);
            }

            // 画像ファイルの最終修正日時
            ExifTag::ModifyDate(s) => if let Some(ndt) = parse_datetime(s) {
                self.exif_capture_info.time_info.modify_date = Some(ndt);
                if self.exif_capture_info.time_info.sub_sec_time.is_none() {
                    self.exif_capture_info.time_info.sub_sec_time = Some(0);
                }
            }
            // タイムゾーン情報
            ExifTag::OffsetTime(s) => { self.exif_capture_info.time_info.offset_time = TimeOffset::from_str(s); }
            // ミリ秒以下
            ExifTag::SubSecTime(s) => if let Ok(u) = s.parse() {
                self.exif_capture_info.time_info.sub_sec_time = Some(u);
            }

            // OpticInfo
            // レンズ製造元
            ExifTag::LensMake(s) => { self.exif_capture_info.optic_info.lens_make = Some(s.clone()); }
            // レンズ製品名（型番）
            ExifTag::LensModel(s) => { self.exif_capture_info.optic_info.lens_model = Some(s.clone()); }
            // レンズの個体識別番号
            ExifTag::LensSerialNumber(s) => { self.exif_capture_info.optic_info.lens_serial_number = Some(s.clone()); }
            // レンズの光学仕様：レンズの最大開放F値（＝最小のFNumber）
            ExifTag::MaxApertureValue(v) => { self.exif_capture_info.optic_info.max_aperture_value = pick_v0_ur64(v); }

            // ExposureSettings
            // 露出制御の方針（プログラムモード）
            // 露出プログラムの種類
            ExifTag::ExposureProgram(v) => { self.exif_capture_info.exposure_settings.exposure_program = Some(ExposureProgram::from_vec(v)); }
            // 実際の撮影者の操作方法
            ExifTag::ExposureMode(v) => { self.exif_capture_info.exposure_settings.exposure_mode = Some(ExposureMode::from_vec(v)); }
            
            // シャッター速度関連
            // 実際のシャッター開放時間（秒） = 物理的な値
            ExifTag::ExposureTime(v) => { self.exif_capture_info.exposure_settings.exposure_time = pick_v0_ur64(v); }
            // シャッター速度のLog2表現（Apex値） = 計算された値：ShutterSpeedValue = -log2(ExposureTime)で更新
            ExifTag::ShutterSpeedValue(v) => { self.exif_capture_info.exposure_settings.shutter_speed_value = pick_v0_ir64(v); }
            
            // 絞り（F値）関連
            // 実際の絞り値 = 光学的な実測値
            ExifTag::FNumber(v) => { self.exif_capture_info.exposure_settings.f_number = pick_v0_ur64(v); }
            // Log2ベースのApex値：A = 2 * log2(FNumber)で更新
            ExifTag::ApertureValue(v) => { self.exif_capture_info.exposure_settings.aperture_value = pick_v0_ur64(v); }

            // 露出補正・明るさ（被写体輝度）
            // カメラが意図的に露出を+/-補正した量（Apex単位）
            ExifTag::ExposureCompensation(v) => { self.exif_capture_info.exposure_settings.exposure_compensation = pick_v0_ir64(v); }
            // 被写体の平均輝度（Apex単位、推定値）
            ExifTag::BrightnessValue(v) => { self.exif_capture_info.exposure_settings.brightness_value = pick_v0_ir64(v); }

            // 露出決定に影響する外部要因
            // 露出計測の方式
            ExifTag::MeteringMode(v) => { self.exif_capture_info.exposure_settings.metering_mode = Some(MeteringMode::from_vec(v)); }
            // 撮影時の光源タイプ
            ExifTag::LightSource(v) => { self.exif_capture_info.exposure_settings.light_source = Some(LightSource::from_vec(v)); }
            // フラッシュの発光状況
            ExifTag::Flash(v) => { self.exif_capture_info.exposure_settings.flash = Some(Flash::from_vec(v)); }

            // 撮影構図・被写体関連
            // レンズの焦点距離（mm）
            ExifTag::FocalLength(v) => { self.exif_capture_info.exposure_settings.focal_length = pick_v0_ur64(v); }
            // フォーカスされた領域の位置とサイズ：長さ2, 3, 4
            ExifTag::SubjectArea(v) => { self.exif_capture_info.exposure_settings.subject_area = Some(v.clone()); }
            // ピントが合った被写体の中心座標（2D）：長さ2
            ExifTag::SubjectLocation(v) => { self.exif_capture_info.exposure_settings.subject_location = pick_v02(v); }
            
            // SensitivityInfo
            // ISO系タグの選択ルール
            ExifTag::SensitivityType(v) => { self.exif_capture_info.sensitivity_info.sensitivity_type = Some(SensitivityType::from_vec(v)); }

            // ISO互換タグ群
            // カメラが設定したISO感度（Exif 2.2以前：主流）：長さは何でもあり
            ExifTag::ISO(v) => { self.exif_capture_info.sensitivity_info.iso = Some(v.clone()); }
            // カメラが設定したISO感度（Exif 2.3以降）
            ExifTag::ISOSpeed(v) => { self.exif_capture_info.sensitivity_info.iso_speed = pick_v0(v); }

            // 感度定義に関する派生タグ群
            // 標準出力感度
            ExifTag::StandardOutputSensitivity(v) => { self.exif_capture_info.sensitivity_info.standard_output_sensitivity = pick_v0(v); }
            // 推奨露出指数（REI）
            ExifTag::RecommendedExposureIndex(v) => { self.exif_capture_info.sensitivity_info.recommended_exposure_index = pick_v0(v); }
            // 実際に使用された感度指数
            ExifTag::ExposureIndex(v) => { self.exif_capture_info.sensitivity_info.exposure_index = pick_v0_ur64(v); }

            // ISO Latitude表現（フィルム互換性向け）
            // フィルムにおける露光許容範囲の「下限」感度
            ExifTag::ISOSpeedLatitudeyyy(v) => { self.exif_capture_info.sensitivity_info.iso_speed_latitude_yyy = pick_v0(v); }
            // フィルムにおける露光許容範囲の「上限」感度
            ExifTag::ISOSpeedLatitudezzz(v) => { self.exif_capture_info.sensitivity_info.iso_speed_latitude_zzz = pick_v0(v); }
            
            // EncodingMetadata
            // 基本仕様・形式情報（Exif構造のメタ情報）
            // Exif仕様のバージョン：長さ4をAscii化
            ExifTag::ExifVersion(v) => { self.exif_capture_info.encoding_metadata.exif_version = v.to_string(); }
            // Flashpix規格バージョン：長さ4をAscii化
            ExifTag::FlashpixVersion(v) => { self.exif_capture_info.encoding_metadata.flashpix_version = v.to_string(); }
            // Exif IFD（画像情報）へのポインタ
            ExifTag::ExifOffset(v) => { self.exif_capture_info.encoding_metadata.exif_offset = pick_v0(v); }
            // RGB/BGRなどのカラーチャネルの順序：長さ4
            ExifTag::ComponentsConfiguration(v) => { self.exif_capture_info.encoding_metadata.components_configuration = load_components_configuration(v); }
            // 圧縮された1ピクセルあたりの平均ビット数
            ExifTag::CompressedBitsPerPixel(v) => { self.exif_capture_info.encoding_metadata.compressed_bits_per_pixel = pick_v0_ur64(v); }

            // センサ・入力系仕様
            // 撮像方式
            ExifTag::SensingMethod(v) => { self.exif_capture_info.encoding_metadata.sensing_method = Some(SensingMethod::from_vec(v)); }
            // ファイルの生成元
            ExifTag::FileSource(v) => { self.exif_capture_info.encoding_metadata.file_source = Some(FileSource::from_u8_vec(v)); }
            // どのような方法で画像が生成されたか
            ExifTag::SceneType(v) => { self.exif_capture_info.encoding_metadata.scene_type = Some(SceneType::from_u8_vec(v)); }
            // ベイヤー配列などのカラー配列パターン
            ExifTag::CFAPattern(v) => { self.exif_capture_info.encoding_metadata.cfa_pattern = CFAPattern::from_vec(v); }

            // 撮影環境情報（自然環境の計測値）
            // 撮影時の気温（℃）
            ExifTag::AmbientTemperature(v) => { self.exif_capture_info.encoding_metadata.ambient_temperature = pick_v0_ir64(v); }
            // 湿度（%）
            ExifTag::Humidity(v) => { self.exif_capture_info.encoding_metadata.humidity = pick_v0_ur64(v); }
            // 気圧（hPa）
            ExifTag::Pressure(v) => { self.exif_capture_info.encoding_metadata.pressure = pick_v0_ur64(v); }
            // 水深（メートル：水中撮影など）
            ExifTag::WaterDepth(v) => { self.exif_capture_info.encoding_metadata.water_depth = pick_v0_ir64(v); }
            // 撮影時の加速度（車載カメラなど）
            ExifTag::Acceleration(v) => { self.exif_capture_info.encoding_metadata.acceleration = pick_v0_ur64(v); }
            // カメラの仰角（水平基準の角度）
            ExifTag::CameraElevationAngle(v) => { self.exif_capture_info.encoding_metadata.camera_elevation_angle = pick_v0_ir64(v); }

            // 光学・測距関連情報
            // 撮影素子の分光感度特性
            ExifTag::SpectralSensitivity(s) => { self.exif_capture_info.encoding_metadata.spectral_sensitivity = Some(s.clone()); }
            // 入力→出力の変換特性（センサの直線性）
            ExifTag::OECF(v) => { self.exif_capture_info.encoding_metadata.oecf = Some(v.clone()); }
            // 被写体までの距離（メートル）
            ExifTag::SubjectDistance(v) => { self.exif_capture_info.encoding_metadata.subject_distance = pick_v0_ur64(v); }
            // 被写体の距離カテゴリ
            ExifTag::SubjectDistanceRange(v) => { self.exif_capture_info.encoding_metadata.subject_distance_range = pick_v0(v); }
            // フラッシュの発光エネルギー
            ExifTag::FlashEnergy(v) => { self.exif_capture_info.encoding_metadata.flash_energy = pick_v0_ur64(v); }
            // シャープネス指標
            ExifTag::SpatialFrequencyResponse(v) => { self.exif_capture_info.encoding_metadata.spatial_frequency_response = Some(v.clone()); }

            // 解像度情報
            // 撮像素子上の水平方向の解像度
            ExifTag::FocalPlaneXResolution(v) => { self.exif_capture_info.encoding_metadata.focal_plane_x_resolution = pick_v0_ur64(v); }
            // 撮像素子上の垂直方向の解像度
            ExifTag::FocalPlaneYResolution(v) => { self.exif_capture_info.encoding_metadata.focal_plane_y_resolution = pick_v0_ur64(v); }
            // 上記の単位
            ExifTag::FocalPlaneResolutionUnit(v) => { self.exif_capture_info.encoding_metadata.focal_plane_resolution_unit = Some(FocalPlaneResolutionUnit::from_vec(v)); }

            // 撮影設定パラメータ
            // ホワイトバランス
            ExifTag::WhiteBalance(v) => { self.exif_capture_info.encoding_metadata.white_balance = Some(WhiteBalance::from_vec(v)); }
            // デジタルズーム倍率
            ExifTag::DigitalZoomRatio(v) => { self.exif_capture_info.encoding_metadata.digital_zoom_ratio = pick_v0_ur64(v); }
            // 35mm換算焦点距離（mm）
            ExifTag::FocalLengthIn35mmFormat(v) => { self.exif_capture_info.encoding_metadata.focal_length_in_35mm_format = pick_v0(v); }
            // 撮影シーン
            ExifTag::SceneCaptureType(v) => { self.exif_capture_info.encoding_metadata.scene_capture_type = Some(SceneCaptureType::from_vec(v)); }
            // ゲイン調整
            ExifTag::GainControl(v) => { self.exif_capture_info.encoding_metadata.gain_control = Some(GainControl::from_vec(v)); }
            // 画像のコントラスト設定
            ExifTag::Contrast(v) => { self.exif_capture_info.encoding_metadata.contrast = Some(Contrast::from_vec(v)); }
            // 彩度設定
            ExifTag::Saturation(v) => { self.exif_capture_info.encoding_metadata.saturation = Some(Saturation::from_vec(v)); }
            // シャープネス設定
            ExifTag::Sharpness(v) => { self.exif_capture_info.encoding_metadata.sharpness = Some(Sharpness::from_vec(v)); }
            // カスタム画像処理の有無（ソフト補正など）
            ExifTag::CustomRendered(v) => { self.exif_capture_info.encoding_metadata.custom_rendered = Some(CustomRendered::from_vec(v)); }
            // 構造化されたカメラ設定
            ExifTag::DeviceSettingDescription(v) => { self.exif_capture_info.encoding_metadata.device_setting_description = Some(v.clone()); }
            // ガンマ補正値
            ExifTag::Gamma(v) => { self.exif_capture_info.encoding_metadata.gamma = pick_v0_ur64(v); }

            // その他
            // 関連する音声ファイル名（撮影時の音声メモなど）
            ExifTag::RelatedSoundFile(s) => { self.exif_capture_info.encoding_metadata.related_sound_file = Some(s.clone()); }
            
            // IdentifierInfo
            // 画像ファイルの識別子（ID）
            ExifTag::ImageUniqueID(s) => { self.exif_capture_info.identifier_info.image_unique_id = Some(s.clone()); }

            // CompositeMetadata
            // この画像が複数画像の合成（合成写真）であるかどうか
            ExifTag::CompositeImage(v) => { self.exif_capture_info.composite_metadata.composite_image = Some(CompositeImage::from_vec(v)); }
            // 何枚の画像から合成されたか
            ExifTag::CompositeImageCount(v) => { self.exif_capture_info.composite_metadata.composite_image_count = pick_v02(v); }
            // 合成元となった各画像の露出時間一覧：長さがCompositeImageCount
            ExifTag::CompositeImageExposureTimes(v) => { self.exif_capture_info.composite_metadata.composite_image_exposure_times = Some(v.clone()); }
            // ExifCaptureInfo（撮影関連情報）（ここまで）

            // GpsInfo（位置情報）（ここから）
            // LocationInfo
            // 現在置：GPS座標・高度
            // 緯度
            ExifTag::GPSLatitudeRef(s) => { self.gps_info.location_info.gps_latitude_ref = gps_ref(s); }
            ExifTag::GPSLatitude(v) => {
                self.gps_info.location_info.gps_latitude = DMS::from_vec(v);
                if self.gps_info.location_info.gps_latitude_ref.is_none() { self.gps_info.location_info.gps_latitude_ref = Some(true); }
            }
            // 経度
            ExifTag::GPSLongitudeRef(s) => { self.gps_info.location_info.gps_longitude_ref = gps_ref(s); }
            ExifTag::GPSLongitude(v) => {
                self.gps_info.location_info.gps_longitude = DMS::from_vec(v);
                if self.gps_info.location_info.gps_longitude_ref.is_none() { self.gps_info.location_info.gps_longitude_ref = Some(true); }
            }
            // 使用されている測地系
            ExifTag::GPSMapDatum(s) => { self.gps_info.location_info.gps_map_datum = Some(s.to_string()); }
            // 高度
            ExifTag::GPSAltitudeRef(v) => { self.gps_info.location_info.gps_altitude_ref = Some(GPSAltitudeRef::from_u8_vec(v)); }
            ExifTag::GPSAltitude(v) => {
                self.gps_info.location_info.gps_altitude = pick_v0_ur64(v);
                if self.gps_info.location_info.gps_altitude_ref.is_none() { self.gps_info.location_info.gps_altitude_ref = Some(GPSAltitudeRef::AboveSeaLevel); }
            }

            // 測位補助情報：衛星・精度・方式
            // 使用衛星情報
            ExifTag::GPSSatellites(s) => { self.gps_info.location_info.gps_satellites = Some(s.to_string()); }
            // 測位の状態
            ExifTag::GPSStatus(s) => { self.gps_info.location_info.gps_status = Some(GPSStatus::from_str(s)); }
            // 2D/3D測位
            ExifTag::GPSMeasureMode(s) => { self.gps_info.location_info.gps_measure_mode = Some(GPSMeasureMode::from_str(s)); }
            // 測位精度（Dilution of Precision）
            ExifTag::GPSDOP(v) => { self.gps_info.location_info.gps_dop = pick_v0_ur64(v); }

            // 運動情報（速度・進行方向）
            // 移動速度
            ExifTag::GPSSpeedRef(s) => { self.gps_info.location_info.gps_speed_ref = Some(GPSSpeedRef::from_str(s)); }
            ExifTag::GPSSpeed(v) => {
                self.gps_info.location_info.gps_speed = pick_v0_ur64(v);
                if self.gps_info.location_info.gps_speed_ref.is_none() { self.gps_info.location_info.gps_speed_ref = Some(GPSSpeedRef::km_h); }
            }
            // 移動方向
            ExifTag::GPSTrackRef(s) => { self.gps_info.location_info.gps_track_ref = Some(NorthRef::from_str(s)); }
            ExifTag::GPSTrack(v) => {
                self.gps_info.location_info.gps_track = pick_v0_ur64(v);
                if self.gps_info.location_info.gps_track_ref.is_none() { self.gps_info.location_info.gps_track_ref = Some(NorthRef::MagneticNorth); }
            }
            // カメラの向き
            ExifTag::GPSImgDirectionRef(s) => { self.gps_info.location_info.gps_img_direction_ref = Some(NorthRef::from_str(s)); }
            ExifTag::GPSImgDirection(v) => {
                self.gps_info.location_info.gps_img_direction = pick_v0_ur64(v);
                if self.gps_info.location_info.gps_img_direction_ref.is_none() { self.gps_info.location_info.gps_img_direction_ref = Some(NorthRef::MagneticNorth); }
            }
            
            // 目的地情報：緯度経度を基準に、相互に逆算できる必要あり
            // 目的地の緯度
            ExifTag::GPSDestLatitudeRef(s) => { self.gps_info.location_info.gps_dest_latitude_ref = gps_ref(s); }
            ExifTag::GPSDestLatitude(v) => {
                self.gps_info.location_info.gps_dest_latitude = DMS::from_vec(v);
                if self.gps_info.location_info.gps_dest_latitude_ref.is_none() { self.gps_info.location_info.gps_dest_latitude_ref = Some(true); }
            }
            // 目的地の経度
            ExifTag::GPSDestLongitudeRef(s) => { self.gps_info.location_info.gps_dest_longitude_ref = gps_ref(s); }
            ExifTag::GPSDestLongitude(v) => {
                self.gps_info.location_info.gps_dest_longitude = DMS::from_vec(v);
                if self.gps_info.location_info.gps_dest_longitude_ref.is_none() { self.gps_info.location_info.gps_dest_longitude_ref = Some(true); }
            }
            // 目的地の方角
            ExifTag::GPSDestBearingRef(s) => { self.gps_info.location_info.gps_dest_bearing_ref = Some(NorthRef::from_str(s)); }
            ExifTag::GPSDestBearing(v) => {
                self.gps_info.location_info.gps_dest_bearing = pick_v0_ur64(v);
                if self.gps_info.location_info.gps_dest_bearing_ref.is_none() { self.gps_info.location_info.gps_dest_bearing_ref = Some(NorthRef::MagneticNorth); }
            }
            // 目的地までの距離
            ExifTag::GPSDestDistanceRef(s) => { self.gps_info.location_info.gps_dest_distance_ref = Some(GPSDestDistanceRef::from_str(s)); }
            ExifTag::GPSDestDistance(v) => {
                self.gps_info.location_info.gps_dest_distance = pick_v0_ur64(v);
                if self.gps_info.location_info.gps_dest_distance_ref.is_none() { self.gps_info.location_info.gps_dest_distance_ref = Some(GPSDestDistanceRef::Kilimeters); }
            }

            // 管理情報：測位記録のメタ情報
            // 測地方法
            ExifTag::GPSProcessingMethod(v) => { self.gps_info.location_info.gps_processing_method = Some(v.clone()); }
            // 地域情報テキスト
            ExifTag::GPSAreaInformation(v) => { self.gps_info.location_info.gps_area_information = Some(v.clone()); }
            // UTC日付
            ExifTag::GPSDateStamp(s) => {
                self.gps_info.location_info.gps_date_stamp = parse_date(s);
                if self.gps_info.location_info.gps_time_stamp.is_none() { self.gps_info.location_info.gps_time_stamp = Some(Local::now().time()); }
            }
            // UTC時刻（時分秒）
            ExifTag::GPSTimeStamp(v) => {
                self.gps_info.location_info.gps_time_stamp = parse_time(v);
                if self.gps_info.location_info.gps_date_stamp.is_none() { self.gps_info.location_info.gps_date_stamp = Some(Local::now().date_naive()); }
            }
            // 差分GPS補正の有無
            ExifTag::GPSDifferential(v) => { self.gps_info.location_info.gps_differential = Some(GPSDifferential::from_vec(v)); }
            // 水平方向の誤差推定
            ExifTag::GPSHPositioningError(v) => { self.gps_info.location_info.gps_h_positioning_error = pick_v0_ur64(v); }
            // GPSのバージョン：長さ4、表示は「0.1.2.3」
            ExifTag::GPSVersionID(v) => { self.gps_info.location_info.gps_version_id = pick_v04(v); }
            // Exifへのオフセットポインタ：ポインタなので16進数で表示
            ExifTag::GPSInfo(v) => { self.gps_info.location_info.gps_info = pick_v0(v); }
            // GpsInfo（位置情報）（ここまで）

            // InteropInfo（互換性情報）（ここから）
            // InteroperabilityInfo
            // InteropIFDへのポインタ
            ExifTag::InteropOffset(v) => { self.interop_info.interop_offset = pick_v0(v); }
            // InteropIFDの識別子
            ExifTag::InteroperabilityIndex(s) => { self.interop_info.interoperability_index = Some(InteroperabilityIndex::from_str(s)); }
            // InteropIFDのバージョン番号
            ExifTag::InteroperabilityVersion(v) => { self.interop_info.interoperability_version = pick_v04(v); }
            // InteropInfo（互換性情報）（ここまで）

            // ThumbnailInfo（サムネイル画像情報）（ここから）
            // 1st ImageFormat
            // サムネイル画像の先頭位置（バイトオフセット）
            ExifTag::ThumbnailOffset(v0, v1) => { self.thumbnail_info.thumbnail_offset = Some((v0.clone(), v1.clone())); }
            // サムネイル画像のサイズ（バイト数）
            ExifTag::ThumbnailLength(v) => { self.thumbnail_info.thumbnail_length = pick_v0(v); }
            // ThumbnailInfo（サムネイル画像情報）（ここまで）

            // UserMetadata（ユーザ記述情報）（ここから）
            // UserInfo
            // 画像の簡潔な説明
            ExifTag::ImageDescription(s) => { self.user_info.image_description = Some(s.to_string()); }
            // 撮影者や著作権者の名前
            ExifTag::Artist(s) => { self.user_info.artist = Some(s.to_string()); }
            // 著作権表記
            ExifTag::Copyright(s) => { self.user_info.copyright = Some(s.to_string()); }
            // ユーザ自由記述欄（備考）
            ExifTag::UserComment(v) => { self.user_info.user_comment = UserComment::from_vec(v); }
            // UserMetadata（ユーザ記述情報）（ここまで）
            
            // UnEditable
            ExifTag::MakerNote(v) => { self.uneditable.maker_note = Some(format!("{:?}", v)); }
            ExifTag::UnknownINT8U(v, hex, group) => { self.uneditable.unknown_dict.insert((group.clone(), *hex), string_number(v)); },
            ExifTag::UnknownSTRING(s, hex, group) => { self.uneditable.unknown_dict.insert((group.clone(), *hex), s.to_string()); }
            ExifTag::UnknownINT16U(v, hex, group) => { self.uneditable.unknown_dict.insert((group.clone(), *hex), string_number(v)); }
            ExifTag::UnknownINT32U(v, hex, group) => { self.uneditable.unknown_dict.insert((group.clone(), *hex), string_number(v)); }
            ExifTag::UnknownRATIONAL64U(v, hex, group) => { self.uneditable.unknown_dict.insert((group.clone(), *hex), string_rational(v)); }
            ExifTag::UnknownINT8S(v, hex, group) => { self.uneditable.unknown_dict.insert((group.clone(), *hex), string_number(v)); }
            ExifTag::UnknownUNDEF(v, hex, group) => { self.uneditable.unknown_dict.insert((group.clone(), *hex), string_number(v)); }
            ExifTag::UnknownINT16S(v, hex, group) => { self.uneditable.unknown_dict.insert((group.clone(), *hex), string_number(v)); }
            ExifTag::UnknownINT32S(v, hex, group) => { self.uneditable.unknown_dict.insert((group.clone(), *hex), string_number(v)); }
            ExifTag::UnknownRATIONAL64S(v, hex, group) => { self.uneditable.unknown_dict.insert((group.clone(), *hex), string_rational(v)); }
            ExifTag::UnknownFLOAT(v, hex, group) => { self.uneditable.unknown_dict.insert((group.clone(), *hex), string_number(v)); }
            ExifTag::UnknownDOUBLE(v, hex, group) => { self.uneditable.unknown_dict.insert((group.clone(), *hex), string_number(v)); }
        }
        self.metadata.set_tag(tag);
    }

    pub fn delete_tag(&mut self, tag: ExifTag) {
        match &tag {
            ExifTag::Make(_) => { self.basic_image_info.device_model.make = None; }
            ExifTag::Model(_) => { self.basic_image_info.device_model.model = None; }
            ExifTag::Software(_) => { self.basic_image_info.device_model.software = None; }

            ExifTag::ImageWidth(_) => { self.basic_image_info.image_format.image_width = None; }
            ExifTag::ImageHeight(_) => { self.basic_image_info.image_format.image_height = None; }
            ExifTag::ExifImageWidth(_) => { self.basic_image_info.image_format.exif_image_width = None; }
            ExifTag::ExifImageHeight(_) => { self.basic_image_info.image_format.exif_image_height = None; }

            ExifTag::XResolution(_) => { self.basic_image_info.image_format.x_resolution = None; }
            ExifTag::YResolution(_) => { self.basic_image_info.image_format.y_resolution = None; }
            ExifTag::ResolutionUnit(_) => { self.basic_image_info.image_format.resolution_unit = None; }

            ExifTag::Compression(_) => { self.basic_image_info.image_format.compression = None; }
            ExifTag::PhotometricInterpretation(_) => { self.basic_image_info.image_format.photometric_interpretation = None; }
            ExifTag::ColorSpace(_) => { self.basic_image_info.image_format.color_space = None; },

            ExifTag::BitsPerSample(_) => { self.basic_image_info.image_format.bits_per_sample = None; }
            ExifTag::SamplesPerPixel(_) => { self.basic_image_info.image_format.samples_per_pixel = None; }
            ExifTag::PlanarConfiguration(_) => { self.basic_image_info.image_format.planar_configuration = None; }
            ExifTag::YCbCrSubSampling(_) => { self.basic_image_info.image_format.ycbcr_sub_sampling = None; }
            ExifTag::YCbCrPositioning(_) => { self.basic_image_info.image_format.ycbcr_positioning = None; }
            ExifTag::YCbCrCoefficients(_) => { self.basic_image_info.image_format.ycbcr_coefficients = None; }

            ExifTag::TransferFunction(_) => { self.basic_image_info.image_format.transfer_function = None; }
            ExifTag::WhitePoint(_) => { self.basic_image_info.image_format.white_point = None; }
            ExifTag::PrimaryChromaticities(_) => { self.basic_image_info.image_format.primary_chromaticities = None; }
            ExifTag::ReferenceBlackWhite(_) => { self.basic_image_info.image_format.reference_black_white = None; }
            ExifTag::ColorMap(_) => { self.basic_image_info.image_format.color_map = None; }

            ExifTag::StripOffsets(_, _) => { self.basic_image_info.image_format.strip_offsets = None; }
            ExifTag::StripByteCounts(_) => { self.basic_image_info.image_format.strip_byte_counts = None; }
            ExifTag::RowsPerStrip(_) => { self.basic_image_info.image_format.rows_per_strip = None; }

            ExifTag::Orientation(_) => { self.basic_image_info.image_format.orientation = None; }
            ExifTag::CellWidth(_) => { self.basic_image_info.image_format.cell_width = None; }
            ExifTag::CellHeight(_) => { self.basic_image_info.image_format.cell_height = None; }
                    
            ExifTag::SerialNumber(_) => { self.basic_image_info.device_info.serial_number = None; }
            ExifTag::OwnerName(_) => { self.basic_image_info.device_info.owner_name = None; }
            ExifTag::LensInfo(_) => { self.basic_image_info.device_info.lens_info = None; }
            
            ExifTag::DateTimeOriginal(_) => { self.exif_capture_info.time_info.date_time_original = None; }
            ExifTag::OffsetTimeOriginal(_) => { self.exif_capture_info.time_info.offset_time_original = None; }
            ExifTag::SubSecTimeOriginal(_) => { self.exif_capture_info.time_info.sub_sec_time_original = None; }

            ExifTag::CreateDate(_) => { self.exif_capture_info.time_info.create_date = None; }
            ExifTag::OffsetTimeDigitized(_) => { self.exif_capture_info.time_info.offset_time_digitized = None; }
            ExifTag::SubSecTimeDigitized(_) => { self.exif_capture_info.time_info.sub_sec_time_digitized = None; }

            ExifTag::ModifyDate(_) => { self.exif_capture_info.time_info.modify_date = None; }
            ExifTag::OffsetTime(_) => { self.exif_capture_info.time_info.offset_time = None; }
            ExifTag::SubSecTime(_) => { self.exif_capture_info.time_info.sub_sec_time = None; }

            ExifTag::LensMake(_) => { self.exif_capture_info.optic_info.lens_make = None; }
            ExifTag::LensModel(_) => { self.exif_capture_info.optic_info.lens_model = None; }
            ExifTag::LensSerialNumber(_) => { self.exif_capture_info.optic_info.lens_serial_number = None; }
            ExifTag::MaxApertureValue(_) => { self.exif_capture_info.optic_info.max_aperture_value = None; }

            ExifTag::ExposureProgram(_) => { self.exif_capture_info.exposure_settings.exposure_program = None; }
            ExifTag::ExposureMode(_) => { self.exif_capture_info.exposure_settings.exposure_mode = None; }
                    
            ExifTag::ExposureTime(_) => { self.exif_capture_info.exposure_settings.exposure_time = None; }
            ExifTag::ShutterSpeedValue(_) => { self.exif_capture_info.exposure_settings.shutter_speed_value = None; }
                    
            ExifTag::FNumber(_) => { self.exif_capture_info.exposure_settings.f_number = None; }
            ExifTag::ApertureValue(_) => { self.exif_capture_info.exposure_settings.aperture_value = None; }

            ExifTag::ExposureCompensation(_) => { self.exif_capture_info.exposure_settings.exposure_compensation = None; }
            ExifTag::BrightnessValue(_) => { self.exif_capture_info.exposure_settings.brightness_value = None; }

            ExifTag::MeteringMode(_) => { self.exif_capture_info.exposure_settings.metering_mode = None; }
            ExifTag::LightSource(_) => { self.exif_capture_info.exposure_settings.light_source = None; }
            ExifTag::Flash(_) => { self.exif_capture_info.exposure_settings.flash = None; }

            ExifTag::FocalLength(_) => { self.exif_capture_info.exposure_settings.focal_length = None; }
            ExifTag::SubjectArea(_) => { self.exif_capture_info.exposure_settings.subject_area = None; }
            ExifTag::SubjectLocation(_) => { self.exif_capture_info.exposure_settings.subject_location = None; }
                    
            ExifTag::SensitivityType(_) => { self.exif_capture_info.sensitivity_info.sensitivity_type = None; }

            ExifTag::ISO(_) => { self.exif_capture_info.sensitivity_info.iso = None; }
            ExifTag::ISOSpeed(_) => { self.exif_capture_info.sensitivity_info.iso_speed = None; }

            ExifTag::StandardOutputSensitivity(_) => { self.exif_capture_info.sensitivity_info.standard_output_sensitivity = None; }
            ExifTag::RecommendedExposureIndex(_) => { self.exif_capture_info.sensitivity_info.recommended_exposure_index = None; }
            ExifTag::ExposureIndex(_) => { self.exif_capture_info.sensitivity_info.exposure_index = None; }

            ExifTag::ISOSpeedLatitudeyyy(_) => { self.exif_capture_info.sensitivity_info.iso_speed_latitude_yyy = None; }
            ExifTag::ISOSpeedLatitudezzz(_) => { self.exif_capture_info.sensitivity_info.iso_speed_latitude_zzz = None; }
                    
            ExifTag::ExifVersion(_) => { self.exif_capture_info.encoding_metadata.exif_version = None; }
            ExifTag::FlashpixVersion(_) => { self.exif_capture_info.encoding_metadata.flashpix_version = None; }
            ExifTag::ExifOffset(_) => { self.exif_capture_info.encoding_metadata.exif_offset = None; }
            ExifTag::ComponentsConfiguration(_) => { self.exif_capture_info.encoding_metadata.components_configuration = None; }
            ExifTag::CompressedBitsPerPixel(_) => { self.exif_capture_info.encoding_metadata.compressed_bits_per_pixel = None; }

            ExifTag::SensingMethod(_) => { self.exif_capture_info.encoding_metadata.sensing_method = None; }
            ExifTag::FileSource(_) => { self.exif_capture_info.encoding_metadata.file_source = None; }
            ExifTag::SceneType(_) => { self.exif_capture_info.encoding_metadata.scene_type = None; }
            ExifTag::CFAPattern(_) => { self.exif_capture_info.encoding_metadata.cfa_pattern = None; }

            ExifTag::AmbientTemperature(_) => { self.exif_capture_info.encoding_metadata.ambient_temperature = None; }
            ExifTag::Humidity(_) => { self.exif_capture_info.encoding_metadata.humidity = None; }
            ExifTag::Pressure(_) => { self.exif_capture_info.encoding_metadata.pressure = None; }
            ExifTag::WaterDepth(_) => { self.exif_capture_info.encoding_metadata.water_depth = None; }
            ExifTag::Acceleration(_) => { self.exif_capture_info.encoding_metadata.acceleration = None; }
            ExifTag::CameraElevationAngle(_) => { self.exif_capture_info.encoding_metadata.camera_elevation_angle = None; }

            ExifTag::SpectralSensitivity(_) => { self.exif_capture_info.encoding_metadata.spectral_sensitivity = None; }
            ExifTag::OECF(_) => { self.exif_capture_info.encoding_metadata.oecf = None; }
            ExifTag::SubjectDistance(_) => { self.exif_capture_info.encoding_metadata.subject_distance = None; }
            ExifTag::SubjectDistanceRange(_) => { self.exif_capture_info.encoding_metadata.subject_distance_range = None; }
            ExifTag::FlashEnergy(_) => { self.exif_capture_info.encoding_metadata.flash_energy = None; }
            ExifTag::SpatialFrequencyResponse(_) => { self.exif_capture_info.encoding_metadata.spatial_frequency_response = None; }

            ExifTag::FocalPlaneXResolution(_) => { self.exif_capture_info.encoding_metadata.focal_plane_x_resolution = None; }
            ExifTag::FocalPlaneYResolution(_) => { self.exif_capture_info.encoding_metadata.focal_plane_y_resolution = None; }
            ExifTag::FocalPlaneResolutionUnit(_) => { self.exif_capture_info.encoding_metadata.focal_plane_resolution_unit = None; }

            ExifTag::WhiteBalance(_) => { self.exif_capture_info.encoding_metadata.white_balance = None; }
            ExifTag::DigitalZoomRatio(_) => { self.exif_capture_info.encoding_metadata.digital_zoom_ratio = None; }
            ExifTag::FocalLengthIn35mmFormat(_) => { self.exif_capture_info.encoding_metadata.focal_length_in_35mm_format = None; }
            ExifTag::SceneCaptureType(_) => { self.exif_capture_info.encoding_metadata.scene_capture_type = None; }
            ExifTag::GainControl(_) => { self.exif_capture_info.encoding_metadata.gain_control = None; }
            ExifTag::Contrast(_) => { self.exif_capture_info.encoding_metadata.contrast = None; }
            ExifTag::Saturation(_) => { self.exif_capture_info.encoding_metadata.saturation = None; }
            ExifTag::Sharpness(_) => { self.exif_capture_info.encoding_metadata.sharpness = None; }
            ExifTag::CustomRendered(_) => { self.exif_capture_info.encoding_metadata.custom_rendered = None; }
            ExifTag::DeviceSettingDescription(_) => { self.exif_capture_info.encoding_metadata.device_setting_description = None; }
            ExifTag::Gamma(_) => { self.exif_capture_info.encoding_metadata.gamma = None; }

            ExifTag::RelatedSoundFile(_) => { self.exif_capture_info.encoding_metadata.related_sound_file = None; }
                    
            ExifTag::ImageUniqueID(_) => { self.exif_capture_info.identifier_info.image_unique_id = None; }

            ExifTag::CompositeImage(_) => { self.exif_capture_info.composite_metadata.composite_image = None; }
            ExifTag::CompositeImageCount(_) => { self.exif_capture_info.composite_metadata.composite_image_count = None; }
            ExifTag::CompositeImageExposureTimes(_) => { self.exif_capture_info.composite_metadata.composite_image_exposure_times = None; }
            
            ExifTag::GPSLatitudeRef(_) => { self.gps_info.location_info.gps_latitude_ref = None; }
            ExifTag::GPSLatitude(_) => { self.gps_info.location_info.gps_latitude = None; }
            ExifTag::GPSLongitudeRef(_) => { self.gps_info.location_info.gps_longitude_ref = None; }
            ExifTag::GPSLongitude(_) => { self.gps_info.location_info.gps_longitude = None; }
            ExifTag::GPSMapDatum(_) => { self.gps_info.location_info.gps_map_datum = None; }
            ExifTag::GPSAltitudeRef(_) => { self.gps_info.location_info.gps_altitude_ref = None; }
            ExifTag::GPSAltitude(_) => { self.gps_info.location_info.gps_altitude = None; }

            ExifTag::GPSSatellites(_) => { self.gps_info.location_info.gps_satellites = None; }
            ExifTag::GPSStatus(_) => { self.gps_info.location_info.gps_status = None; }
            ExifTag::GPSMeasureMode(_) => { self.gps_info.location_info.gps_measure_mode = None; }
            ExifTag::GPSDOP(_) => { self.gps_info.location_info.gps_dop = None; }

            ExifTag::GPSSpeedRef(_) => { self.gps_info.location_info.gps_speed_ref = None; }
            ExifTag::GPSSpeed(_) => { self.gps_info.location_info.gps_speed = None; }
            ExifTag::GPSTrackRef(_) => { self.gps_info.location_info.gps_track_ref = None; }
            ExifTag::GPSTrack(_) => { self.gps_info.location_info.gps_track = None; }
            ExifTag::GPSImgDirectionRef(_) => { self.gps_info.location_info.gps_img_direction_ref = None; }
            ExifTag::GPSImgDirection(_) => { self.gps_info.location_info.gps_img_direction = None; }
                    
            ExifTag::GPSDestLatitudeRef(_) => { self.gps_info.location_info.gps_dest_latitude_ref = None; }
            ExifTag::GPSDestLatitude(_) => { self.gps_info.location_info.gps_dest_latitude = None; }
            ExifTag::GPSDestLongitudeRef(_) => { self.gps_info.location_info.gps_dest_longitude_ref = None; }
            ExifTag::GPSDestLongitude(_) => { self.gps_info.location_info.gps_dest_longitude = None; }
            ExifTag::GPSDestBearingRef(_) => { self.gps_info.location_info.gps_dest_bearing_ref = None; }
            ExifTag::GPSDestBearing(_) => { self.gps_info.location_info.gps_dest_bearing = None; }
            ExifTag::GPSDestDistanceRef(_) => { self.gps_info.location_info.gps_dest_distance_ref = None; }
            ExifTag::GPSDestDistance(_) => { self.gps_info.location_info.gps_dest_distance = None; }

            ExifTag::GPSProcessingMethod(_) => { self.gps_info.location_info.gps_processing_method = None; }
            ExifTag::GPSAreaInformation(_) => { self.gps_info.location_info.gps_area_information = None; }
            ExifTag::GPSDateStamp(_) => { self.gps_info.location_info.gps_date_stamp = None; }
            ExifTag::GPSTimeStamp(_) => { self.gps_info.location_info.gps_time_stamp = None; }
            ExifTag::GPSDifferential(_) => { self.gps_info.location_info.gps_differential = None; }
            ExifTag::GPSHPositioningError(_) => { self.gps_info.location_info.gps_h_positioning_error = None; }
            ExifTag::GPSVersionID(_) => { self.gps_info.location_info.gps_version_id = None; }
            ExifTag::GPSInfo(_) => { self.gps_info.location_info.gps_info = None; }
            
            ExifTag::InteropOffset(_) => { self.interop_info.interop_offset = None; }
            ExifTag::InteroperabilityIndex(_) => { self.interop_info.interoperability_index = None; }
            ExifTag::InteroperabilityVersion(_) => { self.interop_info.interoperability_version = None; }
            
            ExifTag::ThumbnailOffset(_, _) => { self.thumbnail_info.thumbnail_offset = None; }
            ExifTag::ThumbnailLength(_) => { self.thumbnail_info.thumbnail_length = None; }
            
            ExifTag::ImageDescription(_) => { self.user_info.image_description = None; }
            ExifTag::Artist(_) => { self.user_info.artist = None; }
            ExifTag::Copyright(_) => { self.user_info.copyright = None; }
            ExifTag::UserComment(_) => { self.user_info.user_comment = None; }
            
            ExifTag::MakerNote(_) => { self.uneditable.maker_note = None; }
            ExifTag::UnknownINT8U(_, hex, group) => { let _ = self.uneditable.unknown_dict.remove(&(group.clone(), *hex)); }
            ExifTag::UnknownSTRING(_, hex, group) => { let _ = self.uneditable.unknown_dict.remove(&(group.clone(), *hex)); }
            ExifTag::UnknownINT16U(_, hex, group) => { let _ = self.uneditable.unknown_dict.remove(&(group.clone(), *hex)); }
            ExifTag::UnknownINT32U(_, hex, group) => { let _ = self.uneditable.unknown_dict.remove(&(group.clone(), *hex)); }
            ExifTag::UnknownRATIONAL64U(_, hex, group) => { let _ = self.uneditable.unknown_dict.remove(&(group.clone(), *hex)); }
            ExifTag::UnknownINT8S(_, hex, group) => { let _ = self.uneditable.unknown_dict.remove(&(group.clone(), *hex)); }
            ExifTag::UnknownUNDEF(_, hex, group) => { let _ = self.uneditable.unknown_dict.remove(&(group.clone(), *hex)); }
            ExifTag::UnknownINT16S(_, hex, group) => { let _ = self.uneditable.unknown_dict.remove(&(group.clone(), *hex)); }
            ExifTag::UnknownINT32S(_, hex, group) => { let _ = self.uneditable.unknown_dict.remove(&(group.clone(), *hex)); }
            ExifTag::UnknownRATIONAL64S(_, hex, group) => { let _ = self.uneditable.unknown_dict.remove(&(group.clone(), *hex)); }
            ExifTag::UnknownFLOAT(_, hex, group) => { let _ = self.uneditable.unknown_dict.remove(&(group.clone(), *hex)); }
            ExifTag::UnknownDOUBLE(_, hex, group) => { let _ = self.uneditable.unknown_dict.remove(&(group.clone(), *hex)); }
        }
        self.metadata.get_ifd_mut(tag.get_group(), 0).remove_tag(tag);
    }

    pub fn pick_value(&self, tag: ExifTag) -> Option<String> {
        match &tag {
            ExifTag::Make(_) => self.basic_image_info.device_model.make.clone(),
            ExifTag::Model(_) => self.basic_image_info.device_model.model.clone(),
            ExifTag::Software(_) => self.basic_image_info.device_model.software.clone(),

            ExifTag::ImageWidth(_) => match self.basic_image_info.image_format.image_width { Some(v) => Some(v.show_value()), None => None },
            ExifTag::ImageHeight(_) => match self.basic_image_info.image_format.image_height { Some(v) => Some(v.show_value()), None => None },
            ExifTag::ExifImageWidth(_) => match self.basic_image_info.image_format.exif_image_width { Some(v) => Some(v.show_value()), None => None },
            ExifTag::ExifImageHeight(_) => match self.basic_image_info.image_format.exif_image_height { Some(v) => Some(v.show_value()), None => None },

            ExifTag::XResolution(_) => match self.basic_image_info.image_format.x_resolution { Some(v) => Some(v.show_value()), None => None },
            ExifTag::YResolution(_) => match self.basic_image_info.image_format.y_resolution { Some(v) => Some(v.show_value()), None => None },
            ExifTag::ResolutionUnit(_) => match &self.basic_image_info.image_format.resolution_unit { Some(v) => Some(v.show_value()), None => None },

            ExifTag::Compression(_) => match &self.basic_image_info.image_format.compression { Some(v) => Some(v.show_value()), None => None },
            ExifTag::PhotometricInterpretation(_) => match &self.basic_image_info.image_format.photometric_interpretation { Some(v) => Some(v.show_value()), None => None },
            ExifTag::ColorSpace(_) => match &self.basic_image_info.image_format.color_space { Some(v) => Some(v.show_value()), None => None },

            ExifTag::BitsPerSample(_) => match &self.basic_image_info.image_format.bits_per_sample { Some(v) => Some(v.show_value()), None => None },
            ExifTag::SamplesPerPixel(_) => match self.basic_image_info.image_format.samples_per_pixel { Some(v) => Some(v.show_value()), None => None },
            ExifTag::PlanarConfiguration(_) => match &self.basic_image_info.image_format.planar_configuration { Some(v) => Some(v.show_value()), None => None },
            ExifTag::YCbCrSubSampling(_) => match &self.basic_image_info.image_format.ycbcr_sub_sampling { Some(v) => Some(v.show_value()), None => None },
            ExifTag::YCbCrPositioning(_) => match &self.basic_image_info.image_format.ycbcr_positioning { Some(v) => Some(v.show_value()), None => None },
            ExifTag::YCbCrCoefficients(_) => match self.basic_image_info.image_format.ycbcr_coefficients { Some(v) => Some(v.show_value()), None => None },

            ExifTag::TransferFunction(_) => match self.basic_image_info.image_format.transfer_function { Some(v) => Some(v.show_value()), None => None },
            ExifTag::WhitePoint(_) => match self.basic_image_info.image_format.white_point { Some(v) => Some(v.show_value()), None => None },
            ExifTag::PrimaryChromaticities(_) => match self.basic_image_info.image_format.primary_chromaticities { Some(v) => Some(v.show_value()), None => None },
            ExifTag::ReferenceBlackWhite(_) => match self.basic_image_info.image_format.reference_black_white { Some(v) => Some(v.show_value()), None => None },
            ExifTag::ColorMap(_) => match &self.basic_image_info.image_format.color_map { Some(v) => Some(v.show_value()), None => None },

            ExifTag::StripOffsets(_, _) => match &self.basic_image_info.image_format.strip_offsets { Some(v) => Some(v.0.show_value()), None => None },
            ExifTag::StripByteCounts(_) => match &self.basic_image_info.image_format.strip_byte_counts { Some(v) => Some(v.show_value()), None => None },
            ExifTag::RowsPerStrip(_) => match self.basic_image_info.image_format.rows_per_strip { Some(v) => Some(v.show_value()), None => None },

            ExifTag::Orientation(_) => match &self.basic_image_info.image_format.orientation { Some(v) => Some(v.show_value()), None => None },
            ExifTag::CellWidth(_) => match self.basic_image_info.image_format.cell_width { Some(v) => Some(v.show_value()), None => None },
            ExifTag::CellHeight(_) => match self.basic_image_info.image_format.cell_height { Some(v) => Some(v.show_value()), None => None },
                    
            ExifTag::SerialNumber(_) => self.basic_image_info.device_info.serial_number.clone(),
            ExifTag::OwnerName(_) => self.basic_image_info.device_info.owner_name.clone(),
            ExifTag::LensInfo(_) => match self.basic_image_info.device_info.lens_info { Some(v) => Some(v.show_value()), None => None },
            
            ExifTag::DateTimeOriginal(_) => None,
            ExifTag::OffsetTimeOriginal(_) => match &self.exif_capture_info.time_info.offset_time_original { Some(v) => Some(v.show_value()), None => None },
            ExifTag::SubSecTimeOriginal(_) => match self.exif_capture_info.time_info.sub_sec_time_original { Some(v) => Some(v.show_value()), None => None },

            ExifTag::CreateDate(_) => None,
            ExifTag::OffsetTimeDigitized(_) => match &self.exif_capture_info.time_info.offset_time_digitized { Some(v) => Some(v.show_value()), None => None },
            ExifTag::SubSecTimeDigitized(_) => match self.exif_capture_info.time_info.sub_sec_time_digitized { Some(v) => Some(v.show_value()), None => None },

            ExifTag::ModifyDate(_) => None,
            ExifTag::OffsetTime(_) => match &self.exif_capture_info.time_info.offset_time { Some(v) => Some(v.show_value()), None => None },
            ExifTag::SubSecTime(_) => match self.exif_capture_info.time_info.sub_sec_time { Some(v) => Some(v.show_value()), None => None },

            ExifTag::LensMake(_) => self.exif_capture_info.optic_info.lens_make.clone(),
            ExifTag::LensModel(_) => self.exif_capture_info.optic_info.lens_model.clone(),
            ExifTag::LensSerialNumber(_) => self.exif_capture_info.optic_info.lens_serial_number.clone(),
            ExifTag::MaxApertureValue(_) => match self.exif_capture_info.optic_info.max_aperture_value { Some(v) => Some(v.show_value()), None => None },

            ExifTag::ExposureProgram(_) => match &self.exif_capture_info.exposure_settings.exposure_program { Some(v) => Some(v.show_value()), None => None },
            ExifTag::ExposureMode(_) => match &self.exif_capture_info.exposure_settings.exposure_mode { Some(v) => Some(v.show_value()), None => None },
                    
            ExifTag::ExposureTime(_) => match self.exif_capture_info.exposure_settings.exposure_time { Some(v) => Some(v.show_value()), None => None },
            ExifTag::ShutterSpeedValue(_) => match self.exif_capture_info.exposure_settings.shutter_speed_value { Some(v) => Some(v.show_value()), None => None },
                    
            ExifTag::FNumber(_) => match self.exif_capture_info.exposure_settings.f_number { Some(v) => Some(v.show_value()), None => None },
            ExifTag::ApertureValue(_) => match self.exif_capture_info.exposure_settings.aperture_value { Some(v) => Some(v.show_value()), None => None },

            ExifTag::ExposureCompensation(_) => match self.exif_capture_info.exposure_settings.exposure_compensation { Some(v) => Some(v.show_value()), None => None },
            ExifTag::BrightnessValue(_) => match self.exif_capture_info.exposure_settings.brightness_value { Some(v) => Some(v.show_value()), None => None },

            ExifTag::MeteringMode(_) => match &self.exif_capture_info.exposure_settings.metering_mode { Some(v) => Some(v.show_value()), None => None },
            ExifTag::LightSource(_) => match &self.exif_capture_info.exposure_settings.light_source { Some(v) => Some(v.show_value()), None => None },
            ExifTag::Flash(_) => match &self.exif_capture_info.exposure_settings.flash { Some(v) => Some(v.show_value()), None => None },

            ExifTag::FocalLength(_) => match self.exif_capture_info.exposure_settings.focal_length { Some(v) => Some(v.show_value()), None => None },
            ExifTag::SubjectArea(_) => match &self.exif_capture_info.exposure_settings.subject_area { Some(v) => Some(v.show_value()), None => None },
            ExifTag::SubjectLocation(_) => match self.exif_capture_info.exposure_settings.subject_location { Some(v) => Some(v.show_value()), None => None },
                    
            ExifTag::SensitivityType(_) => match &self.exif_capture_info.sensitivity_info.sensitivity_type { Some(v) => Some(v.show_value()), None => None },

            ExifTag::ISO(_) => match &self.exif_capture_info.sensitivity_info.iso { Some(v) => Some(v.show_value()), None => None },
            ExifTag::ISOSpeed(_) => match self.exif_capture_info.sensitivity_info.iso_speed { Some(v) => Some(v.show_value()), None => None },

            ExifTag::StandardOutputSensitivity(_) => match self.exif_capture_info.sensitivity_info.standard_output_sensitivity { Some(v) => Some(v.show_value()), None => None },
            ExifTag::RecommendedExposureIndex(_) => match self.exif_capture_info.sensitivity_info.recommended_exposure_index { Some(v) => Some(v.show_value()), None => None },
            ExifTag::ExposureIndex(_) => match self.exif_capture_info.sensitivity_info.exposure_index { Some(v) => Some(v.show_value()), None => None },

            ExifTag::ISOSpeedLatitudeyyy(_) => match self.exif_capture_info.sensitivity_info.iso_speed_latitude_yyy { Some(v) => Some(v.show_value()), None => None },
            ExifTag::ISOSpeedLatitudezzz(_) => match self.exif_capture_info.sensitivity_info.iso_speed_latitude_zzz { Some(v) => Some(v.show_value()), None => None },
                    
            ExifTag::ExifVersion(_) => match &self.exif_capture_info.encoding_metadata.exif_version { Some(v) => Some(v.show_value()), None => None },
            ExifTag::FlashpixVersion(_) => match &self.exif_capture_info.encoding_metadata.flashpix_version { Some(v) => Some(v.show_value()), None => None },
            ExifTag::ExifOffset(_) => match self.exif_capture_info.encoding_metadata.exif_offset { Some(v) => Some(v.show_value()), None => None },
            ExifTag::ComponentsConfiguration(_) => None,
            ExifTag::CompressedBitsPerPixel(_) => match self.exif_capture_info.encoding_metadata.compressed_bits_per_pixel { Some(v) => Some(v.show_value()), None => None },

            ExifTag::SensingMethod(_) => match &self.exif_capture_info.encoding_metadata.sensing_method { Some(v) => Some(v.show_value()), None => None },
            ExifTag::FileSource(_) => match &self.exif_capture_info.encoding_metadata.file_source { Some(v) => Some(v.show_value()), None => None },
            ExifTag::SceneType(_) => match &self.exif_capture_info.encoding_metadata.scene_type { Some(v) => Some(v.show_value()), None => None },
            ExifTag::CFAPattern(_) => None,

            ExifTag::AmbientTemperature(_) => match self.exif_capture_info.encoding_metadata.ambient_temperature { Some(v) => Some(v.show_value()), None => None },
            ExifTag::Humidity(_) => match self.exif_capture_info.encoding_metadata.humidity { Some(v) => Some(v.show_value()), None => None },
            ExifTag::Pressure(_) => match self.exif_capture_info.encoding_metadata.pressure { Some(v) => Some(v.show_value()), None => None },
            ExifTag::WaterDepth(_) => match self.exif_capture_info.encoding_metadata.water_depth { Some(v) => Some(v.show_value()), None => None },
            ExifTag::Acceleration(_) => match self.exif_capture_info.encoding_metadata.acceleration { Some(v) => Some(v.show_value()), None => None },
            ExifTag::CameraElevationAngle(_) => match self.exif_capture_info.encoding_metadata.camera_elevation_angle { Some(v) => Some(v.show_value()), None => None },

            ExifTag::SpectralSensitivity(_) => self.exif_capture_info.encoding_metadata.spectral_sensitivity.clone(),
            ExifTag::OECF(_) => match &self.exif_capture_info.encoding_metadata.oecf { Some(v) => Some(v.show_value()), None => None },
            ExifTag::SubjectDistance(_) => match self.exif_capture_info.encoding_metadata.subject_distance { Some(v) => Some(v.show_value()), None => None },
            ExifTag::SubjectDistanceRange(_) => match self.exif_capture_info.encoding_metadata.subject_distance_range { Some(v) => Some(v.show_value()), None => None },
            ExifTag::FlashEnergy(_) => match self.exif_capture_info.encoding_metadata.flash_energy { Some(v) => Some(v.show_value()), None => None },
            ExifTag::SpatialFrequencyResponse(_) => match &self.exif_capture_info.encoding_metadata.spatial_frequency_response { Some(v) => Some(v.show_value()), None => None },

            ExifTag::FocalPlaneXResolution(_) => match self.exif_capture_info.encoding_metadata.focal_plane_x_resolution { Some(v) => Some(v.show_value()), None => None },
            ExifTag::FocalPlaneYResolution(_) => match self.exif_capture_info.encoding_metadata.focal_plane_y_resolution { Some(v) => Some(v.show_value()), None => None },
            ExifTag::FocalPlaneResolutionUnit(_) => match &self.exif_capture_info.encoding_metadata.focal_plane_resolution_unit { Some(v) => Some(v.show_value()), None => None },

            ExifTag::WhiteBalance(_) => match &self.exif_capture_info.encoding_metadata.white_balance { Some(v) => Some(v.show_value()), None => None },
            ExifTag::DigitalZoomRatio(_) => match self.exif_capture_info.encoding_metadata.digital_zoom_ratio { Some(v) => Some(v.show_value()), None => None },
            ExifTag::FocalLengthIn35mmFormat(_) => match self.exif_capture_info.encoding_metadata.focal_length_in_35mm_format { Some(v) => Some(v.show_value()), None => None },
            ExifTag::SceneCaptureType(_) => match &self.exif_capture_info.encoding_metadata.scene_capture_type { Some(v) => Some(v.show_value()), None => None },
            ExifTag::GainControl(_) => match &self.exif_capture_info.encoding_metadata.gain_control { Some(v) => Some(v.show_value()), None => None },
            ExifTag::Contrast(_) => match &self.exif_capture_info.encoding_metadata.contrast { Some(v) => Some(v.show_value()), None => None },
            ExifTag::Saturation(_) => match &self.exif_capture_info.encoding_metadata.saturation { Some(v) => Some(v.show_value()), None => None },
            ExifTag::Sharpness(_) => match &self.exif_capture_info.encoding_metadata.sharpness { Some(v) => Some(v.show_value()), None => None },
            ExifTag::CustomRendered(_) => match &self.exif_capture_info.encoding_metadata.custom_rendered { Some(v) => Some(v.show_value()), None => None },
            ExifTag::DeviceSettingDescription(_) => match &self.exif_capture_info.encoding_metadata.device_setting_description { Some(v) => Some(v.show_value()), None => None },
            ExifTag::Gamma(_) => match self.exif_capture_info.encoding_metadata.gamma { Some(v) => Some(v.show_value()), None => None },

            ExifTag::RelatedSoundFile(_) => self.exif_capture_info.encoding_metadata.related_sound_file.clone(),
                    
            ExifTag::ImageUniqueID(_) => self.exif_capture_info.identifier_info.image_unique_id.clone(),

            ExifTag::CompositeImage(_) => match &self.exif_capture_info.composite_metadata.composite_image { Some(v) => Some(v.show_value()), None => None },
            ExifTag::CompositeImageCount(_) => match self.exif_capture_info.composite_metadata.composite_image_count { Some(v) => Some(v.show_value()), None => None },
            ExifTag::CompositeImageExposureTimes(_) => match &self.exif_capture_info.composite_metadata.composite_image_exposure_times { Some(v) => Some(v.show_value()), None => None },
            
            ExifTag::GPSLatitudeRef(_) => match self.gps_info.location_info.gps_latitude_ref { Some(v) => Some(v.to_string()), None => None },
            ExifTag::GPSLatitude(_) => None,
            ExifTag::GPSLongitudeRef(_) => match self.gps_info.location_info.gps_latitude_ref { Some(v) => Some(v.to_string()), None => None },
            ExifTag::GPSLongitude(_) => None,
            ExifTag::GPSMapDatum(_) => self.gps_info.location_info.gps_map_datum.clone(),
            ExifTag::GPSAltitudeRef(_) => match &self.gps_info.location_info.gps_altitude_ref { Some(v) => Some(v.show_value()), None => None },
            ExifTag::GPSAltitude(_) => match self.gps_info.location_info.gps_altitude { Some(v) => Some(v.show_value()), None => None },

            ExifTag::GPSSatellites(_) => self.gps_info.location_info.gps_satellites.clone(),
            ExifTag::GPSStatus(_) => match &self.gps_info.location_info.gps_status { Some(v) => Some(v.show_value()), None => None },
            ExifTag::GPSMeasureMode(_) => match &self.gps_info.location_info.gps_measure_mode { Some(v) => Some(v.show_value()), None => None },
            ExifTag::GPSDOP(_) => match self.gps_info.location_info.gps_dop { Some(v) => Some(v.show_value()), None => None },

            ExifTag::GPSSpeedRef(_) => match &self.gps_info.location_info.gps_speed_ref { Some(v) => Some(v.show_value()), None => None },
            ExifTag::GPSSpeed(_) => match self.gps_info.location_info.gps_speed { Some(v) => Some(v.show_value()), None => None },
            ExifTag::GPSTrackRef(_) => match &self.gps_info.location_info.gps_track_ref { Some(v) => Some(v.show_value()), None => None },
            ExifTag::GPSTrack(_) => match self.gps_info.location_info.gps_track { Some(v) => Some(v.show_value()), None => None },
            ExifTag::GPSImgDirectionRef(_) => match &self.gps_info.location_info.gps_img_direction_ref { Some(v) => Some(v.show_value()), None => None },
            ExifTag::GPSImgDirection(_) => match self.gps_info.location_info.gps_img_direction { Some(v) => Some(v.show_value()), None => None },
                    
            ExifTag::GPSDestLatitudeRef(_) => match self.gps_info.location_info.gps_latitude_ref { Some(v) => Some(v.to_string()), None => None },
            ExifTag::GPSDestLatitude(_) => None,
            ExifTag::GPSDestLongitudeRef(_) => match self.gps_info.location_info.gps_latitude_ref { Some(v) => Some(v.to_string()), None => None },
            ExifTag::GPSDestLongitude(_) => None,
            ExifTag::GPSDestBearingRef(_) => match &self.gps_info.location_info.gps_dest_bearing_ref { Some(v) => Some(v.show_value()), None => None },
            ExifTag::GPSDestBearing(_) => match self.gps_info.location_info.gps_dest_bearing { Some(v) => Some(v.show_value()), None => None },
            ExifTag::GPSDestDistanceRef(_) => match &self.gps_info.location_info.gps_dest_distance_ref { Some(v) => Some(v.show_value()), None => None },
            ExifTag::GPSDestDistance(_) => match self.gps_info.location_info.gps_dest_distance { Some(v) => Some(v.show_value()), None => None },

            ExifTag::GPSProcessingMethod(_) => match &self.gps_info.location_info.gps_processing_method { Some(v) => Some(v.show_value()), None => None },
            ExifTag::GPSAreaInformation(_) => match &self.gps_info.location_info.gps_area_information { Some(v) => Some(v.show_value()), None => None },
            ExifTag::GPSDateStamp(_) => None,
            ExifTag::GPSTimeStamp(_) => None,
            ExifTag::GPSDifferential(_) => match &self.gps_info.location_info.gps_differential { Some(v) => Some(v.show_value()), None => None },
            ExifTag::GPSHPositioningError(_) => match self.gps_info.location_info.gps_h_positioning_error { Some(v) => Some(v.show_value()), None => None },
            ExifTag::GPSVersionID(_) => match self.gps_info.location_info.gps_version_id { Some(v) => Some(v.show_value()), None => None },
            ExifTag::GPSInfo(_) => match self.gps_info.location_info.gps_info { Some(v) => Some(v.show_value()), None => None },
            
            ExifTag::InteropOffset(_) => match self.interop_info.interop_offset { Some(v) => Some(v.show_value()), None => None },
            ExifTag::InteroperabilityIndex(_) => match &self.interop_info.interoperability_index { Some(v) => Some(v.show_value()), None => None },
            ExifTag::InteroperabilityVersion(_) => match self.interop_info.interoperability_version { Some(v) => Some(v.show_value()), None => None },
            
            ExifTag::ThumbnailOffset(_, _) => match &self.thumbnail_info.thumbnail_offset { Some(v) => Some(v.0.show_value()), None => None },
            ExifTag::ThumbnailLength(_) => match self.thumbnail_info.thumbnail_length { Some(v) => Some(v.show_value()), None => None },
            
            ExifTag::ImageDescription(_) => self.user_info.image_description.clone(),
            ExifTag::Artist(_) => self.user_info.artist.clone(),
            ExifTag::Copyright(_) => self.user_info.copyright.clone(),
            ExifTag::UserComment(_) => match &self.user_info.user_comment { Some(v) => Some(v.decoded.clone()), None => None },
            
            ExifTag::MakerNote(_) => self.uneditable.maker_note.clone(),
            _ => None,
        }
    }
}

fn string_number<T: fmt::Debug + fmt::Display>(v: &[T]) -> String {
    match v.len() {
        0 => "None".to_string(),
        1 => format!("{}", v[0]),
        _ => format!("{:?}", v)
    }
}

fn string_rational<T: ExifRational>(v: &[T]) -> String {
    match v.len() {
        0 => "None".to_string(),
        1 => v[0].to_string(),
        _ => format!("{:?}", v.iter().map(|vi| vi.to_string()).collect::<Vec<String>>())
    }
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