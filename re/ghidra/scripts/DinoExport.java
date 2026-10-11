// Post-analysis: make sure every collected entry is a function, then export decompiled C for all functions.
import ghidra.app.decompiler.*;
import ghidra.app.script.GhidraScript;
import ghidra.program.model.address.Address;
import ghidra.program.model.listing.*;
import java.io.*;
import java.nio.file.*;

public class DinoExport extends GhidraScript {
    @Override
    public void run() throws Exception {
        String[] args = getScriptArgs();
        int created = 0;
        for (String line : Files.readAllLines(Paths.get(args[0]))) {
            line = line.trim();
            if (line.isEmpty()) continue;
            Address a = toAddr(Long.parseLong(line, 16));
            if (getFunctionAt(a) != null) continue;
            if (getInstructionAt(a) == null) disassemble(a);
            if (getInstructionAt(a) != null && createFunction(a, null) != null) created++;
        }
        println("DinoExport: created " + created + " extra functions");

        DecompInterface di = new DecompInterface();
        DecompileOptions opts = new DecompileOptions();
        di.setOptions(opts);
        di.toggleCCode(true);
        di.toggleSyntaxTree(false);
        di.openProgram(currentProgram);
        int ok = 0, fail = 0;
        try (PrintWriter pw = new PrintWriter(new BufferedWriter(new FileWriter(args[1])));
             PrintWriter idx = new PrintWriter(new BufferedWriter(new FileWriter(args[1] + ".index.tsv")))) {
            FunctionIterator it = currentProgram.getFunctionManager().getFunctions(true);
            while (it.hasNext() && !monitor.isCancelled()) {
                Function f = it.next();
                DecompileResults r = di.decompileFunction(f, 60, monitor);
                pw.println("// ==== " + f.getEntryPoint() + " " + f.getName() + " size=" + f.getBody().getNumAddresses());
                if (r != null && r.decompileCompleted()) {
                    pw.println(r.getDecompiledFunction().getC());
                    ok++;
                } else {
                    pw.println("// decompile failed: " + (r == null ? "null" : r.getErrorMessage()));
                    fail++;
                }
                idx.println(f.getEntryPoint() + "\t" + f.getName() + "\t" + f.getBody().getNumAddresses()
                        + "\t" + f.getSymbol().getReferenceCount());
            }
        }
        di.dispose();
        println("DinoExport: decompiled " + ok + " ok, " + fail + " failed");
    }
}
