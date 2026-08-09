// crc32.js
const fs = require('fs');
const path = require('path');
const readline = require('readline');
const yargs = require('yargs/yargs');
const { hideBin } = require('yargs/helpers');
const crc32 = require('crc/crc32');
const crc32c = require('crc/crc32c');

function computeCRC32(data, useCRC32c = false) {
    if (useCRC32c) {
        return crc32c(data).toString(16).toUpperCase();
    } else {
        return crc32(data).toString(16).toUpperCase();
    }
}

function computeFileCRC32(filename, useCRC32c = false) {
    return new Promise((resolve, reject) => {
        const stream = fs.createReadStream(filename);
        const crc = useCRC32c ? crc32c : crc32;
        let hash = 0;
        stream.on('data', (chunk) => {
            hash = crc(chunk, hash);
        });
        stream.on('end', () => {
            resolve(hash.toString(16).toUpperCase());
        });
        stream.on('error', reject);
    });
}

function formatHash(hash, format) {
    if (format === 'hex') {
        return `0x${hash.padStart(8, '0')}`;
    } else if (format === 'dec') {
        return parseInt(hash, 16).toString();
    } else if (format === 'bin') {
        return parseInt(hash, 16).toString(2).padStart(32, '0');
    }
    return `0x${hash.padStart(8, '0')}`;
}

async function main() {
    const argv = yargs(hideBin(process.argv))
        .usage('Использование: $0 <строка или файл> [опции]')
        .option('crc32c', { type: 'boolean', description: 'Использовать CRC32C' })
        .option('check', { type: 'string', description: 'Сравнить с хэшем (HEX)' })
        .option('dec', { type: 'boolean', description: 'Вывод в десятичном формате' })
        .option('bin', { type: 'boolean', description: 'Вывод в бинарном формате' })
        .help()
        .parse();

    const inputs = argv._;
    const useCRC32c = argv.crc32c || false;
    const check = argv.check || null;
    let format = 'hex';
    if (argv.dec) format = 'dec';
    if (argv.bin) format = 'bin';

    // Проверка stdin
    if (inputs.length === 0 && !process.stdin.isTTY) {
        const rl = readline.createInterface({
            input: process.stdin,
            output: process.stdout,
            terminal: false
        });
        let data = '';
        for await (const line of rl) {
            data += line + '\n';
        }
        const hash = computeCRC32(Buffer.from(data), useCRC32c);
        console.log(formatHash(hash, format));
        return;
    }

    if (inputs.length === 0) {
        console.error('Не указаны данные.');
        process.exit(1);
    }

    for (const item of inputs) {
        if (fs.existsSync(item) && fs.statSync(item).isFile()) {
            const hash = await computeFileCRC32(item, useCRC32c);
            let output = `${item}: ${formatHash(hash, format)}`;
            if (check) {
                const expected = parseInt(check.startsWith('0x') ? check.slice(2) : check, 16);
                const actual = parseInt(hash, 16);
                output += (expected === actual) ? ' (✅ OK)' : ' (❌ FAIL)';
            }
            console.log(output);
        } else {
            const data = Buffer.from(item, 'utf8');
            const hash = computeCRC32(data, useCRC32c);
            let output = formatHash(hash, format);
            if (check) {
                const expected = parseInt(check.startsWith('0x') ? check.slice(2) : check, 16);
                const actual = parseInt(hash, 16);
                output += (expected === actual) ? ' (✅ OK)' : ' (❌ FAIL)';
            }
            console.log(output);
        }
    }
}

main().catch(console.error);
