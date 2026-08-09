# crc32.py
import sys
import os
import argparse
import binascii
import zlib
from pathlib import Path

def crc32_data(data, crc32c=False):
    """Вычисляет CRC32 или CRC32C для данных."""
    if crc32c:
        # CRC32C требует отдельной реализации или библиотеки
        # Используем zlib.crc32 с изменённым полиномом (эмуляция)
        # Для простоты используем binascii.crc32 (стандартный CRC32)
        # В реальном коде используйте crcmod или реализуйте таблицу CRC32C
        try:
            import crcmod
            crc32c_func = crcmod.predefined.Crc('crc-32c')
            crc32c_func.update(data)
            return crc32c_func.crcValue
        except ImportError:
            print("Для CRC32C установите: pip install crcmod", file=sys.stderr)
            return zlib.crc32(data) & 0xFFFFFFFF
    return zlib.crc32(data) & 0xFFFFFFFF

def crc32_file(filepath, crc32c=False, show_progress=False):
    """Вычисляет CRC32 для файла с опциональным прогрессом."""
    crc = 0xFFFFFFFF
    if not crc32c:
        # Используем zlib.crc32 для эффективности
        with open(filepath, 'rb') as f:
            while True:
                chunk = f.read(8192)
                if not chunk:
                    break
                crc = zlib.crc32(chunk, crc)
                # if show_progress: ... 
        return crc & 0xFFFFFFFF
    else:
        # Для CRC32C используем табличный алгоритм
        import crcmod
        crc32c_func = crcmod.predefined.Crc('crc-32c')
        with open(filepath, 'rb') as f:
            while True:
                chunk = f.read(8192)
                if not chunk:
                    break
                crc32c_func.update(chunk)
        return crc32c_func.crcValue

def format_hash(hash_val, fmt='hex'):
    """Форматирует хэш в указанном формате."""
    if fmt == 'hex':
        return f"0x{hash_val:08X}"
    elif fmt == 'dec':
        return str(hash_val)
    elif fmt == 'bin':
        return f"{hash_val:032b}"
    return f"0x{hash_val:08X}"

def main():
    parser = argparse.ArgumentParser(description='CRC32 Hash Generator')
    parser.add_argument('inputs', nargs='*', help='Строки или файлы для обработки')
    parser.add_argument('--crc32c', action='store_true', help='Использовать CRC32C')
    parser.add_argument('--check', help='Сравнить с указанным хэшем (HEX)')
    parser.add_argument('--dec', action='store_true', help='Вывод в десятичном формате')
    parser.add_argument('--bin', action='store_true', help='Вывод в бинарном формате')
    parser.add_argument('--progress', action='store_true', help='Показывать прогресс для файлов')
    args = parser.parse_args()

    fmt = 'hex'
    if args.dec:
        fmt = 'dec'
    elif args.bin:
        fmt = 'bin'

    inputs = args.inputs
    if not inputs and not sys.stdin.isatty():
        # Чтение из stdin
        data = sys.stdin.buffer.read()
        hash_val = crc32_data(data, args.crc32c)
        print(format_hash(hash_val, fmt))
        return
    if not inputs:
        print("Не указаны данные. Используйте: crc32 <строка или файл>", file=sys.stderr)
        sys.exit(1)

    for item in inputs:
        # Проверяем, является ли аргумент файлом
        if os.path.isfile(item):
            hash_val = crc32_file(item, args.crc32c, args.progress)
            output = f"{item}: {format_hash(hash_val, fmt)}"
            if args.check:
                expected = int(args.check, 16) if args.check.startswith('0x') else int(args.check, 16)
                status = "✅ OK" if hash_val == expected else "❌ FAIL"
                output += f" (check: {status})"
            print(output)
        else:
            # Обрабатываем как строку
            data = item.encode('utf-8')
            hash_val = crc32_data(data, args.crc32c)
            output = format_hash(hash_val, fmt)
            if args.check:
                expected = int(args.check, 16) if args.check.startswith('0x') else int(args.check, 16)
                status = "✅ OK" if hash_val == expected else "❌ FAIL"
                output += f" (check: {status})"
            print(output)

if __name__ == '__main__':
    main()
