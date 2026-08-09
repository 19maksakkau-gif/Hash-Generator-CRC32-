// Crc32.java
import java.io.*;
import java.nio.file.*;
import java.util.zip.CRC32;
import java.util.zip.CRC32C;
import java.util.*;

public class Crc32 {
    public static void main(String[] args) throws Exception {
        boolean useCrc32c = false;
        String check = null;
        boolean dec = false, bin = false;
        List<String> inputs = new ArrayList<>();

        for (int i=0; i<args.length; i++) {
            if (args[i].equals("--crc32c")) {
                useCrc32c = true;
            } else if (args[i].equals("--check") && i+1 < args.length) {
                check = args[++i];
            } else if (args[i].equals("--dec")) {
                dec = true;
            } else if (args[i].equals("--bin")) {
                bin = true;
            } else {
                inputs.add(args[i]);
            }
        }

        String fmt = "hex";
        if (dec) fmt = "dec";
        else if (bin) fmt = "bin";

        // Проверка stdin
        if (inputs.isEmpty() && System.in.available() > 0) {
            ByteArrayOutputStream baos = new ByteArrayOutputStream();
            byte[] buf = new byte[8192];
            int n;
            while ((n = System.in.read(buf)) > 0) {
                baos.write(buf, 0, n);
            }
            byte[] data = baos.toByteArray();
            long hash = computeCrc32(data, useCrc32c);
            System.out.println(formatHash(hash, fmt));
            return;
        }

        if (inputs.isEmpty()) {
            System.err.println("Не указаны данные.");
            System.exit(1);
        }

        for (String item : inputs) {
            Path path = Paths.get(item);
            if (Files.exists(path) && !Files.isDirectory(path)) {
                byte[] data = Files.readAllBytes(path);
                long hash = computeCrc32(data, useCrc32c);
                String output = item + ": " + formatHash(hash, fmt);
                if (check != null) {
                    long expected = Long.parseLong(check.startsWith("0x") ? check.substring(2) : check, 16);
                    output += (expected == hash) ? " (✅ OK)" : " (❌ FAIL)";
                }
                System.out.println(output);
            } else {
                byte[] data = item.getBytes("UTF-8");
                long hash = computeCrc32(data, useCrc32c);
                String output = formatHash(hash, fmt);
                if (check != null) {
                    long expected = Long.parseLong(check.startsWith("0x") ? check.substring(2) : check, 16);
                    output += (expected == hash) ? " (✅ OK)" : " (❌ FAIL)";
                }
                System.out.println(output);
            }
        }
    }

    private static long computeCrc32(byte[] data, boolean useCrc32c) {
        if (useCrc32c) {
            CRC32C crc = new CRC32C();
            crc.update(data);
            return crc.getValue();
        } else {
            CRC32 crc = new CRC32();
            crc.update(data);
            return crc.getValue();
        }
    }

    private static String formatHash(long hash, String fmt) {
        if (fmt.equals("hex")) {
            return String.format("0x%08X", hash);
        } else if (fmt.equals("dec")) {
            return String.valueOf(hash);
        } else if (fmt.equals("bin")) {
            return String.format("%32s", Long.toBinaryString(hash)).replace(' ', '0');
        }
        return String.format("0x%08X", hash);
    }
}
