import * as vscode from 'vscode';
import { execSync, exec } from 'child_process';
import * as path from 'path';
import * as fs from 'fs';

function getNonce(): string {
    let text = '';
    const chars = 'ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789';
    for (let i = 0; i < 32; i++) {
        text += chars.charAt(Math.floor(Math.random() * chars.length));
    }
    return text;
}

class VoltDocument implements vscode.CustomDocument {
    constructor(readonly uri: vscode.Uri) {}
    dispose(): void {}
}

class VoltEditorProvider implements vscode.CustomReadonlyEditorProvider<VoltDocument> {
    constructor(private readonly context: vscode.ExtensionContext) {}

    openCustomDocument(uri: vscode.Uri): VoltDocument {
        return new VoltDocument(uri);
    }

    async resolveCustomEditor(
        document: VoltDocument,
        webviewPanel: vscode.WebviewPanel
    ): Promise<void> {
        webviewPanel.webview.options = { enableScripts: true };
        const data = fs.readFileSync(document.uri.fsPath);
        webviewPanel.webview.html = this.getHtml(data.toString('base64'));
    }

    private getHtml(base64: string): string {
        const nonce = getNonce();
        return `<!DOCTYPE html>
<html>
<head>
<meta charset="UTF-8">
<meta http-equiv="Content-Security-Policy" content="default-src 'none'; script-src 'nonce-${nonce}'; style-src 'unsafe-inline';">
<style>
  body { background: #1e1e1e; display: flex; flex-direction: column; justify-content: center; align-items: center; height: 100vh; margin: 0; }
  canvas { image-rendering: pixelated; max-width: 100%; max-height: 90vh; box-shadow: 0 4px 24px #0008; }
  #info { color: #888; font: 12px monospace; margin-top: 10px; }
  #error { color: #f48771; font: 13px monospace; padding: 20px; white-space: pre; }
</style>
</head>
<body>
<div id="error"></div>
<div id="info"></div>
<script nonce="${nonce}">
(function () {
    const b64 = '${base64}';

    function b64ToBuffer(b64) {
        const bin = atob(b64);
        const buf = new Uint8Array(bin.length);
        for (let i = 0; i < bin.length; i++) buf[i] = bin.charCodeAt(i);
        return buf.buffer;
    }

    function decodeVolt(buf) {
        const v = new DataView(buf);
        let p = 0;

        const magic = String.fromCharCode(v.getUint8(0), v.getUint8(1), v.getUint8(2), v.getUint8(3));
        if (magic !== 'volt') throw new Error('Not a valid .volt file (bad magic)');
        p = 4;

        p++; // version
        const width  = v.getUint16(p, true); p += 2;
        const height = v.getUint16(p, true); p += 2;
        const flags  = v.getUint8(p++);

        const hasAlpha  = (flags & 0x01) !== 0;
        const hasPalette = (flags & 0x02) !== 0;

        let palette = null;
        if (hasPalette) {
            const count = v.getUint8(p++);
            palette = [];
            for (let i = 0; i < count; i++) {
                if (hasAlpha) {
                    palette.push([v.getUint8(p), v.getUint8(p+1), v.getUint8(p+2), v.getUint8(p+3)]);
                    p += 4;
                } else {
                    palette.push([v.getUint8(p), v.getUint8(p+1), v.getUint8(p+2), 255]);
                    p += 3;
                }
            }
        }

        const canvas = document.createElement('canvas');
        canvas.width = width; canvas.height = height;
        const ctx = canvas.getContext('2d');
        const img = ctx.createImageData(width, height);
        const d   = img.data;

        function readColor() {
            if (palette) { return palette[v.getUint8(p++)]; }
            if (hasAlpha) { const c = [v.getUint8(p),v.getUint8(p+1),v.getUint8(p+2),v.getUint8(p+3)]; p+=4; return c; }
            const c = [v.getUint8(p),v.getUint8(p+1),v.getUint8(p+2),255]; p+=3; return c;
        }

        function setPixel(x, y, c) {
            const i = (y * width + x) * 4;
            d[i]=c[0]; d[i+1]=c[1]; d[i+2]=c[2]; d[i+3]=c[3]!==undefined?c[3]:255;
        }

        function fillRect(x, y, w, h, c) {
            for (let row = y; row < y + h; row++)
                for (let col = x; col < x + w; col++)
                    setPixel(col, row, c);
        }

        while (p < buf.byteLength) {
            const op = v.getUint8(p++);
            if (op === 0xFF) break;

            if (op === 0x01) {
                fillRect(0, 0, width, height, readColor());

            } else if (op === 0x02) {
                const rx = v.getUint16(p,true); p+=2;
                const ry = v.getUint16(p,true); p+=2;
                const rw = v.getUint16(p,true); p+=2;
                const rh = v.getUint16(p,true); p+=2;
                fillRect(rx, ry, rw, rh, readColor());

            } else if (op === 0x06) {
                const bx = v.getUint16(p,true); p+=2;
                const by = v.getUint16(p,true); p+=2;
                const bw = v.getUint16(p,true); p+=2;
                const bh = v.getUint16(p,true); p+=2;
                const palType = v.getUint8(p++);
                const dataLen = v.getUint32(p,true); p+=4;
                const end = p + dataLen;

                let cx = bx, cy = by;
                while (p < end) {
                    const count = v.getUint8(p++);
                    let c;
                    if (palType === 0x00 && palette) {
                        c = palette[v.getUint8(p++)];
                    } else if (palType === 0x04) {
                        c = [v.getUint8(p),v.getUint8(p+1),v.getUint8(p+2),v.getUint8(p+3)]; p+=4;
                    } else {
                        c = [v.getUint8(p),v.getUint8(p+1),v.getUint8(p+2),255]; p+=3;
                    }
                    for (let i = 0; i < count; i++) {
                        if (cx >= bx + bw) { cx = bx; cy++; }
                        if (cy >= by + bh) break;
                        setPixel(cx++, cy, c);
                    }
                }
                p = end;
            }
        }

        ctx.putImageData(img, 0, 0);
        return { canvas, width, height, bytes: buf.byteLength };
    }

    try {
        const { canvas, width, height, bytes } = decodeVolt(b64ToBuffer(b64));
        document.body.insertBefore(canvas, document.getElementById('info'));
        document.getElementById('info').textContent =
            width + ' x ' + height + ' px  |  ' + bytes + ' bytes';
    } catch (e) {
        document.getElementById('error').textContent = 'Decode error: ' + e.message;
    }
})();
</script>
</body>
</html>`;
    }
}

export function activate(context: vscode.ExtensionContext) {
    context.subscriptions.push(
        vscode.window.registerCustomEditorProvider(
            'volt.voltPreview',
            new VoltEditorProvider(context),
            { supportsMultipleEditorsPerDocument: false }
        )
    );

    const disposable = vscode.commands.registerCommand('volt.convert', async () => {
        const fileUri = await vscode.window.showOpenDialog({
            canSelectMany: false,
            openLabel: 'Select Image',
            filters: { 'Images': ['png', 'jpg', 'jpeg'] }
        });

        if (!fileUri || !fileUri[0]) { return; }
        const imagePath = fileUri[0].fsPath;

        const isWindows  = process.platform === 'win32';
        const binaryName = isWindows ? 'volt.exe' : 'volt';
        const projectRoot = context.extensionPath;
        const binaryPath  = path.join(projectRoot, 'target', 'debug', binaryName);

        if (!fs.existsSync(binaryPath)) {
            vscode.window.showInformationMessage('Compiling Volt...');
            try {
                execSync('cargo build', { cwd: projectRoot });
            } catch {
                vscode.window.showErrorMessage('Cargo build failed.');
                return;
            }
        }

        const confirm = await vscode.window.showInformationMessage(
            `Convert to .volt:\n${imagePath}`,
            { modal: true },
            'Yes'
        );

        if (confirm !== 'Yes') { return; }

        const command = isWindows
            ? `"${binaryPath}" "${imagePath}"`
            : `${binaryPath} "${imagePath}"`;

        exec(command, { cwd: path.dirname(imagePath) }, (err, stdout, stderr) => {
            if (err) {
                vscode.window.showErrorMessage(`Error: ${stderr || err.message}`);
                return;
            }
            vscode.window.showInformationMessage(stdout.trim());
        });
    });

    context.subscriptions.push(disposable);
}
