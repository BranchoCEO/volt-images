<h1 align="center">HOW TO USE</h1>
<h3 align="left">1.</h3>
<p align="left">
Install "volt-images" on VS code or https://marketplace.visualstudio.com/items?itemName=BranchoCEO.volt-images
</p>

![Screenshot 2026-03-22 084330](https://github.com/user-attachments/assets/779d57f4-f984-4410-b33c-5105b34f4db2)


<h3 align="left">2.</h3>
<p align="left">
In your terminal enter specify the file directory, e.g cd C:\Users\John\Documents\OneDrive\Images
</p>
<pre>
cd C:\Users\John\Documents\OneDrive\Images
</pre>

<h3 align="left">3.</h3>
<p align="left">
Run, e.g volt "image.png" from the file directory
</p>
<pre>
volt "images.png"
</pre>

<h2 align="center">How to use in a HTML script</h2>

<p align="left">Make sure your files look like this<p>
<p align="left">
  <img src="https://github.com/user-attachments/assets/452d4333-9815-4379-b23d-8c10bb79c687">
</p>

<h3 align="left">1.</h3>
<p align="left">Make sure to include "<script src="app.js"></script>"<p>
<pre>
 &lt;!DOCTYPE html&gt;
&lt;html lang=&quot;en&quot;&gt;
&lt;head&gt;
  &lt;meta charset=&quot;UTF-8&quot;&gt;
  &lt;style&gt;
    * { margin: 0; padding: 0; }
    canvas { display: block; }
  &lt;/style&gt;

&lt;/head&gt;
&lt;body&gt;
  &lt;canvas id=&quot;canvas&quot;&gt;&lt;/canvas&gt;
  &lt;script src=&quot;app.js&quot;&gt;&lt;/script&gt;
&lt;/body&gt;
&lt;/html&gt;
</pre>

<h3 align="left">2.</h3>
<p align="left">In your app.js, make SURE to include something like"const response = await fetch('image.txt');"<p>
<pre>
  const canvas = document.getElementById(&#39;canvas&#39;);
const ctx    = canvas.getContext(&#39;2d&#39;);
const info   = document.getElementById(&#39;info&#39;);
const error  = document.getElementById(&#39;error&#39;);

const FLAG_ALPHA   = 0x01;
const FLAG_PALETTE = 0x02;
const OP_FILL_BG   = 0x01;
const OP_RECT      = 0x02;
const OP_RASTER    = 0x06;
const OP_EOF       = 0xFF;
const PAL_GLOBAL   = 0x00;
const PAL_RGBA     = 0x04;

async function main() {
    try {
        const response = await fetch(&#39;image-resources2.volt&#39;);
        if (!response.ok) throw new Error(`HTTP ${response.status}: ${response.statusText}`);
        const buf = await response.arrayBuffer();
        render(buf);
    } catch (e) {
        error.textContent = &#39;Error: &#39; + e.message;
    }
}

function render(buf) {
    const v = new DataView(buf);
    let p = 0;

    const magic = String.fromCharCode(v.getUint8(0), v.getUint8(1), v.getUint8(2), v.getUint8(3));
    if (magic !== &#39;volt&#39;) throw new Error(&#39;Not a valid .volt file (bad magic bytes)&#39;);
    p = 4;

    p++;
    const width  = v.getUint16(p, true); p += 2;
    const height = v.getUint16(p, true); p += 2;
    const flags  = v.getUint8(p++);

    const hasAlpha   = (flags &amp; FLAG_ALPHA)   !== 0;
    const hasPalette = (flags &amp; FLAG_PALETTE) !== 0;

    let palette = null;
    if (hasPalette) {
        const count = v.getUint8(p++);
        palette = [];
        for (let i = 0; i &lt; count; i++) {
            if (hasAlpha) {
                palette.push([v.getUint8(p), v.getUint8(p+1), v.getUint8(p+2), v.getUint8(p+3)]);
                p += 4;
            } else {
                palette.push([v.getUint8(p), v.getUint8(p+1), v.getUint8(p+2), 255]);
                p += 3;
            }
        }
    }

    canvas.width  = width;
    canvas.height = height;
    const img = ctx.createImageData(width, height);
    const d   = img.data;

    function readColor() {
        if (palette) { return palette[v.getUint8(p++)]; }
        if (hasAlpha) { const c = [v.getUint8(p), v.getUint8(p+1), v.getUint8(p+2), v.getUint8(p+3)]; p += 4; return c; }
        const c = [v.getUint8(p), v.getUint8(p+1), v.getUint8(p+2), 255]; p += 3; return c;
    }

    function setPixel(x, y, c) {
        const i = (y * width + x) * 4;
        d[i] = c[0]; d[i+1] = c[1]; d[i+2] = c[2]; d[i+3] = c[3] !== undefined ? c[3] : 255;
    }

    function fillRect(x, y, w, h, c) {
        for (let row = y; row &lt; y + h; row++)
            for (let col = x; col &lt; x + w; col++)
                setPixel(col, row, c);
    }

    while (p &lt; buf.byteLength) {
        const op = v.getUint8(p++);
        if (op === OP_EOF) break;

        if (op === OP_FILL_BG) {
            fillRect(0, 0, width, height, readColor());

        } else if (op === OP_RECT) {
            const rx = v.getUint16(p, true); p += 2;
            const ry = v.getUint16(p, true); p += 2;
            const rw = v.getUint16(p, true); p += 2;
            const rh = v.getUint16(p, true); p += 2;
            fillRect(rx, ry, rw, rh, readColor());

        } else if (op === OP_RASTER) {
            const bx = v.getUint16(p, true); p += 2;
            const by = v.getUint16(p, true); p += 2;
            const bw = v.getUint16(p, true); p += 2;
            const bh = v.getUint16(p, true); p += 2;
            const palType = v.getUint8(p++);
            const dataLen = v.getUint32(p, true); p += 4;
            const end = p + dataLen;

            let cx = bx, cy = by;
            while (p &lt; end) {
                const count = v.getUint8(p++);
                let c;
                if (palType === PAL_GLOBAL &amp;&amp; palette) {
                    c = palette[v.getUint8(p++)];
                } else if (palType === PAL_RGBA) {
                    c = [v.getUint8(p), v.getUint8(p+1), v.getUint8(p+2), v.getUint8(p+3)]; p += 4;
                } else {
                    c = [v.getUint8(p), v.getUint8(p+1), v.getUint8(p+2), 255]; p += 3;
                }
                for (let i = 0; i &lt; count; i++) {
                    if (cx &gt;= bx + bw) { cx = bx; cy++; }
                    if (cy &gt;= by + bh) break;
                    setPixel(cx++, cy, c);
                }
            }
            p = end;
        }
    }

    ctx.putImageData(img, 0, 0);
    info.textContent = `${width} x ${height} px  |  ${buf.byteLength} bytes`;
}

main();
<pre>
<h2 align="center">⚠️MAKE SURE YOU HAVE THE .TXT FILE IN YOUR PROJECT FOLDER⚠️</h2>
