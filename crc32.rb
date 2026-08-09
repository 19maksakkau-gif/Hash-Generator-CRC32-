# crc32.rb
require 'zlib'
require 'optparse'
require 'stringio'

def crc32_data(data, use_crc32c = false)
  if use_crc32c
    # CRC32C не встроен в Ruby, используем собственную реализацию
    crc = 0xFFFFFFFF
    table = generate_crc32c_table
    data.each_byte do |byte|
      crc = table[(crc ^ byte) & 0xFF] ^ (crc >> 8)
    end
    crc ^ 0xFFFFFFFF
  else
    Zlib.crc32(data)
  end
end

def generate_crc32c_table
  poly = 0x1EDC6F41
  table = Array.new(256)
  256.times do |i|
    crc = i
    8.times do
      crc = (crc & 1) != 0 ? (poly ^ (crc >> 1)) : (crc >> 1)
    end
    table[i] = crc
  end
  table
end

def crc32_file(filename, use_crc32c = false)
  File.open(filename, 'rb') do |f|
    data = f.read
    crc32_data(data, use_crc32c)
  end
end

def format_hash(hash, fmt = 'hex')
  case fmt
  when 'hex'
    "0x%08X" % hash
  when 'dec'
    hash.to_s
  when 'bin'
    "%032b" % hash
  else
    "0x%08X" % hash
  end
end

options = {}
OptionParser.new do |opts|
  opts.banner = "Использование: ruby crc32.rb [опции] <строка или файл>"
  opts.on("--crc32c", "Использовать CRC32C") { options[:crc32c] = true }
  opts.on("--check HASH", "Сравнить с хэшем (HEX)") { |v| options[:check] = v }
  opts.on("--dec", "Вывод в десятичном формате") { options[:dec] = true }
  opts.on("--bin", "Вывод в бинарном формате") { options[:bin] = true }
end.parse!

inputs = ARGV
use_crc32c = options[:crc32c] || false
check = options[:check]
fmt = 'hex'
fmt = 'dec' if options[:dec]
fmt = 'bin' if options[:bin]

# Проверка stdin
if inputs.empty?
  data = $stdin.read
  if data && !data.empty?
    hash = crc32_data(data, use_crc32c)
    puts format_hash(hash, fmt)
    exit
  end
end

if inputs.empty?
  $stderr.puts "Не указаны данные."
  exit 1
end

inputs.each do |item|
  if File.file?(item)
    hash = crc32_file(item, use_crc32c)
    output = "#{item}: #{format_hash(hash, fmt)}"
    if check
      expected = check.to_i(16)
      output << (expected == hash ? " (✅ OK)" : " (❌ FAIL)")
    end
    puts output
  else
    data = item.encode('UTF-8')
    hash = crc32_data(data, use_crc32c)
    output = format_hash(hash, fmt)
    if check
      expected = check.to_i(16)
      output << (expected == hash ? " (✅ OK)" : " (❌ FAIL)")
    end
    puts output
  end
end
