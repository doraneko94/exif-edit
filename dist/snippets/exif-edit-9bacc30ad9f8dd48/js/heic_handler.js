export async function ensureJpegBytes(file) {
    try {
        console.log("[Start] ensureJpegBytes for:", file.name, file.type);

        const isHeic = (file.type === "image/heic" || file.name.toLowerCase().endsWith(".heic"));

        if (!isHeic) {
            console.log("[Info] JPEG detected, skipping conversion.");
            const buffer = await file.arrayBuffer();
            return {
                jpeg: new Uint8Array(buffer),
                exif: {}
            }
        }

        // --- 1. Exifメタ全体取得 ---
        console.time("exifr.parse");
        const exifObj = await exifr.parse(file, { translateValues: false });
        console.timeEnd("exifr.parse");
        if (!exifObj) {
            console.warn("[Warn] Exif data could not be parsed from HEIC.");
        } else {
            console.log("[Info] Exif keys:", Object.keys(exifObj));
        }

        // --- 3. HEIC → JPEG変換 ---
        console.time("heic2any");
        const jpegBlob = await heic2any({ blob: file, toType: "image/jpeg" });
        console.timeEnd("heic2any");
        console.log("[Info] JPEG blob size:", jpegBlob.size);

        const buffer = await jpegBlob.arrayBuffer();
        const finalBytes = new Uint8Array(buffer);
        /*let finalBytes;
        if (exifObj?.Orientation && [3, 6, 8].includes(exifObj.Orientation)) {
            console.log("[Info] Applying orientation correction:", exifObj.Orientation);
            finalBytes = await rotateAndConvertToJpeg(jpegBlob, exifObj.Orientation);
        } else {
            const buffer = await jpegBlob.arrayBuffer();
            finalBytes = new Uint8Array(buffer);
        }*/
        return {
            jpeg: finalBytes,
            exif: sanitizeExif(exifObj),
        }
    } catch (err) {
        console.error("[Error] Failed in ensureJpegBytes:", err);
        return {
            jpeg: new Uint8Array([]),
            exif: {}
        }
    }
}

function sanitizeExif(obj) {
    const replacer = (_, value) => {
        if (
            value instanceof Uint8Array ||
            value instanceof Uint16Array ||
            value instanceof Uint32Array ||
            value instanceof Int8Array ||
            value instanceof Int16Array ||
            value instanceof Int32Array ||
            value instanceof Float32Array ||
            value instanceof Float64Array
        ) {
            return Array.from(value);
        }
        return value;
    };
    return JSON.parse(JSON.stringify(obj, replacer));
}

async function rotateAndConvertToJpeg(blob, orientation) {
    const imageBitmap = await createImageBitmap(blob);
    const canvas = document.createElement("canvas");
    const ctx = canvas.getContext("2d");

    // 回転に応じてキャンバスのサイズを変更
    if (orientation === 6 || orientation === 8) {
        canvas.width = imageBitmap.height;
        canvas.height = imageBitmap.width;
    } else {
        canvas.width = imageBitmap.width;
        canvas.height = imageBitmap.height;
    }

    // 回転
    ctx.save();
    switch (orientation) {
        case 3:
            ctx.translate(canvas.width, canvas.height);
            ctx.rotate(Math.PI);
            break;
        case 6:
            ctx.translate(canvas.width, 0);
            ctx.rotate(Math.PI / 2);
            break;
        case 8:
            ctx.translate(0, canvas.height);
            ctx.rotate(-Math.PI / 2);
            break;
        default:
            break;
    }

    ctx.drawImage(imageBitmap, 0, 0);
    ctx.restore();

    return new Promise((resolve) => {
        canvas.toBlob((blob) => {
            blob.arrayBuffer().then(buffer => {
                resolve(new Uint8Array(buffer));
            });
        }, "image/jpeg");
    });
}