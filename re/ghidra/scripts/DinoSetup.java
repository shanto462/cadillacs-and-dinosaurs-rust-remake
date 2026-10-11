// Pre-analysis setup for the CPS1 "dino" 68000 image: A5 global base, entry points, hardware labels.
import ghidra.app.script.GhidraScript;
import ghidra.program.model.address.Address;
import ghidra.program.model.lang.Register;
import ghidra.program.model.symbol.SourceType;
import java.math.BigInteger;
import java.nio.file.*;

public class DinoSetup extends GhidraScript {
    @Override
    public void run() throws Exception {
        Register a5 = currentProgram.getRegister("A5");
        currentProgram.getProgramContext().setValue(a5, toAddr(0), toAddr(0x17FFFF), BigInteger.valueOf(0xFF8000));
        int n = 0;
        for (String line : Files.readAllLines(Paths.get(getScriptArgs()[0]))) {
            line = line.trim();
            if (line.isEmpty()) continue;
            Address a = toAddr(Long.parseLong(line, 16));
            currentProgram.getSymbolTable().addExternalEntryPoint(a);
            n++;
        }
        String[][] labels = {
            {"800000", "IN_P1P2"}, {"800018", "IN_SYSTEM"}, {"80001a", "DIP_A"}, {"80001c", "DIP_B"}, {"80001e", "DIP_C"},
            {"800030", "COIN_CTRL"}, {"800100", "CPSA_OBJ_BASE"}, {"800102", "CPSA_SCROLL1_BASE"}, {"800104", "CPSA_SCROLL2_BASE"},
            {"800106", "CPSA_SCROLL3_BASE"}, {"800108", "CPSA_OTHER_BASE"}, {"80010a", "CPSA_PAL_BASE"},
            {"80010c", "CPSA_SCROLL1_X"}, {"80010e", "CPSA_SCROLL1_Y"}, {"800110", "CPSA_SCROLL2_X"}, {"800112", "CPSA_SCROLL2_Y"},
            {"800114", "CPSA_SCROLL3_X"}, {"800116", "CPSA_SCROLL3_Y"}, {"800120", "CPSA_ROWSCROLL_OFS"}, {"800122", "CPSA_VIDEO_CTRL"},
            {"80014a", "CPSB_LAYER_CTRL"}, {"900000", "OBJ_RAM_A"}, {"904000", "OBJ_RAM_B"}, {"908000", "SCROLL1_RAM"},
            {"90c000", "SCROLL2_RAM"}, {"910000", "SCROLL3_RAM"}, {"914000", "PAL_OBJ"}, {"914400", "PAL_SCROLL1"},
            {"914800", "PAL_SCROLL2"}, {"914c00", "PAL_SCROLL3"}, {"f18000", "QSOUND_SHARED"}, {"f1c001", "IN_P3"},
            {"ff0052", "TASK_TABLE"}, {"ffb274", "PLAYER1"}, {"ffb3f4", "PLAYER2"}, {"ffb574", "PLAYER3"},
            {"ffc8f4", "POOL_A_ENEMIES"}, {"ffb6f4", "POOL_B"}, {"ff9a74", "POOL_C"}, {"ffddf4", "POOL_D"},
            {"ff8744", "CAMERA_X"}, {"ff8748", "CAMERA_Y"}, {"ff8004", "RNG_STATE"}, {"ff8016", "INPUT_P1"}, {"ff8017", "INPUT_P2"},
            {"ff8018", "INPUT_P3"}, {"ff84d9", "STAGE"}, {"ff8786", "AREA"}, {"ff84aa", "DEBUG_MODE"},
            {"8ce", "task_create"}, {"92e", "task_kill"}, {"95a", "task_sleep"}, {"9b6", "task_yield"}, {"9de", "task_suspend"},
            {"9fa", "task_wake"}, {"a76", "sleep_1"}, {"a7a", "sleep_2"}, {"a7e", "sleep_30"}, {"a82", "sleep_60"},
            {"a86", "sleep_120"}, {"a8c", "sleep_180"}, {"a9a", "rng_next"}, {"c0a", "print_string"}, {"41a", "reset"},
            {"542", "vblank_irq"}, {"83c", "task_scheduler"}, {"13fa6", "build_sprite_list"}, {"141f8", "draw_object"},
            {"aaf6", "game_frame"}, {"a69a", "boot_task"}, {"9d048", "system_task"},
        };
        for (String[] l : labels) {
            createLabel(toAddr(Long.parseLong(l[0], 16)), l[1], true, SourceType.USER_DEFINED);
        }
        println("DinoSetup: A5=0xFF8000, " + n + " entry points, " + labels.length + " labels");
    }
}
