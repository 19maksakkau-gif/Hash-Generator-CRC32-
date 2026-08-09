// crc32.cs
using System;
using System.Collections.Generic;
using System.IO;
using System.IO.Hashing;
using System.Text;

class Crc32
{
    static void Main(string[] args)
    {
        bool useCrc32c = false;
        string check = null;
        bool dec = false, bin = false;
        var inputs = new List<string>();

        for (int i=0; i<args.Length; i++)
        {
            switch (args[i])
            {
                case "--crc32c": useCrc32c = true; break;
                case "--check": if (i+1 < args.Length) check = args[++i]; break;
                case "--dec": dec = true; break;
                case "--bin": bin = true; break;
                default: inputs.Add(args[i]); break;
            }
        }

        string fmt = "hex";
        if (dec) fmt = "dec";
        else if (bin) fmt = "bin";

        // Проверка stdin
        if (inputs.Count == 0 && Console.IsInputRedirected)
        {
            using (var ms = new MemoryStream())
            {
                Console.OpenStandardInput().CopyTo(ms);
                byte[] data = ms.ToArray();
                uint hash = ComputeCrc32(data, useCrc32c);
                Console.WriteLine(FormatHash(hash, fmt));
                return;
            }
        }

        if (inputs.Count == 0)
        {
            Console.Error.WriteLine("Не указаны данные.");
            Environment.Exit(1);
        }

        foreach (var item in inputs)
        {
            if (File.Exists(item))
            {
                byte[] data = File.ReadAllBytes(item);
                uint hash = ComputeCrc32(data, useCrc32c);
                string output = $"{item}: {FormatHash(hash, fmt)}";
                if (check != null)
                {
                    uint expected = Convert.ToUInt32(check.StartsWith("0x") ? check.Substring(2) : check, 16);
                    output += (expected == hash) ? " (✅ OK)" : " (❌ FAIL)";
                }
                Console.WriteLine(output);
            }
            else
            {
                byte[] data = Encoding.UTF8.GetBytes(item);
                uint hash = ComputeCrc32(data, useCrc32c);
                string output = FormatHash(hash, fmt);
                if (check != null)
                {
                    uint expected = Convert.ToUInt32(check.StartsWith("0x") ? check.Substring(2) : check, 16);
                    output += (expected == hash) ? " (✅ OK)" : " (❌ FAIL)";
                }
                Console.WriteLine(output);
            }
        }
    }

    static uint ComputeCrc32(byte[] data, bool useCrc32c)
    {
        if (useCrc32c)
        {
            return Crc32C.HashToUInt32(data);
        }
        else
        {
            return System.IO.Hashing.Crc32.HashToUInt32(data);
        }
    }

    static string FormatHash(uint hash, string fmt)
    {
        if (fmt == "hex") return $"0x{hash:X8}";
        if (fmt == "dec") return hash.ToString();
        if (fmt == "bin") return Convert.ToString(hash, 2).PadLeft(32, '0');
        return $"0x{hash:X8}";
    }
}
