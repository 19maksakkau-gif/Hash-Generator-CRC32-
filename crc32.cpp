// crc32.cpp
#include <iostream>
#include <fstream>
#include <string>
#include <vector>
#include <cstdint>
#include <iomanip>
#include <algorithm>
#include <bitset>
#include <cstring>
#include <unistd.h>
#include <sys/stat.h>

using namespace std;

// Таблица CRC32 (IEEE)
static const uint32_t crc32_table[256] = {
    0x00000000, 0x77073096, 0xEE0E612C, 0x990951BA,
    0x076DC419, 0x706AF48F, 0xE963A535, 0x9E6495A3,
    0x0EDB8832, 0x79DCB8A4, 0xE0D5E91E, 0x97D2D988,
    0x09B64C2B, 0x7EB17CBD, 0xE7B82D07, 0x90BF1D91,
    // ... полная таблица опущена для краткости
};

// Упрощённая таблица CRC32C (Castagnoli)
static const uint32_t crc32c_table[256] = {
    0x00000000, 0xF26B8303, 0xE13B70F7, 0x1350F3F4,
    // ... полная таблица опущена для краткости
};

uint32_t crc32_update(uint32_t crc, const uint8_t* data, size_t len, bool crc32c) {
    const uint32_t* table = crc32c ? crc32c_table : crc32_table;
    for (size_t i=0; i<len; ++i) {
        crc = table[(crc ^ data[i]) & 0xFF] ^ (crc >> 8);
    }
    return crc;
}

uint32_t crc32_data(const string& data, bool crc32c) {
    uint32_t crc = 0xFFFFFFFF;
    crc = crc32_update(crc, (const uint8_t*)data.data(), data.size(), crc32c);
    return crc ^ 0xFFFFFFFF;
}

uint32_t crc32_file(const string& filename, bool crc32c) {
    ifstream file(filename, ios::binary);
    if (!file.is_open()) {
        cerr << "Не удалось открыть " << filename << endl;
        return 0;
    }
    uint32_t crc = 0xFFFFFFFF;
    char buffer[8192];
    while (file.read(buffer, sizeof(buffer))) {
        crc = crc32_update(crc, (uint8_t*)buffer, file.gcount(), crc32c);
    }
    crc = crc32_update(crc, (uint8_t*)buffer, file.gcount(), crc32c);
    file.close();
    return crc ^ 0xFFFFFFFF;
}

string format_hash(uint32_t hash, const string& fmt) {
    if (fmt == "hex") {
        stringstream ss;
        ss << "0x" << uppercase << setw(8) << setfill('0') << hex << hash;
        return ss.str();
    } else if (fmt == "dec") {
        return to_string(hash);
    } else if (fmt == "bin") {
        bitset<32> bs(hash);
        return bs.to_string();
    }
    return to_string(hash);
}

int main(int argc, char* argv[]) {
    bool use_crc32c = false;
    string check = "";
    bool dec = false, bin = false;
    vector<string> inputs;

    for (int i=1; i<argc; ++i) {
        string arg = argv[i];
        if (arg == "--crc32c") use_crc32c = true;
        else if (arg == "--check" && i+1 < argc) check = argv[++i];
        else if (arg == "--dec") dec = true;
        else if (arg == "--bin") bin = true;
        else inputs.push_back(arg);
    }

    string fmt = "hex";
    if (dec) fmt = "dec";
    else if (bin) fmt = "bin";

    // Если нет аргументов и есть данные в stdin
    if (inputs.empty()) {
        if (!isatty(STDIN_FILENO)) {
            string data((istreambuf_iterator<char>(cin)), istreambuf_iterator<char>());
            uint32_t hash = crc32_data(data, use_crc32c);
            cout << format_hash(hash, fmt) << endl;
            return 0;
        }
        cerr << "Не указаны данные." << endl;
        return 1;
    }

    for (const string& item : inputs) {
        struct stat st;
        if (stat(item.c_str(), &st) == 0 && S_ISREG(st.st_mode)) {
            uint32_t hash = crc32_file(item, use_crc32c);
            string output = item + ": " + format_hash(hash, fmt);
            if (!check.empty()) {
                uint32_t expected = strtoul(check.c_str(), nullptr, 16);
                output += (expected == hash) ? " (✅ OK)" : " (❌ FAIL)";
            }
            cout << output << endl;
        } else {
            uint32_t hash = crc32_data(item, use_crc32c);
            string output = format_hash(hash, fmt);
            if (!check.empty()) {
                uint32_t expected = strtoul(check.c_str(), nullptr, 16);
                output += (expected == hash) ? " (✅ OK)" : " (❌ FAIL)";
            }
            cout << output << endl;
        }
    }
    return 0;
}
