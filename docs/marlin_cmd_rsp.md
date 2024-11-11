## M20
Begin file list
BOAT~1.GCO 3759599
RABBIT~1.GCO 5137185
CEIL~221.GCO 778893
End file list

## M27
Not SD printing
ok

## M104
:0 B@:0
ok

## M105
ok T:21.17 /0.00 B:20.31 /0.00

## M106
ok

## M107
ok

## M114
X:-13.00 Y:-7.50 Z:10.00 E:0.00 Count X:-1040 Y:-600 Z:4000
ok

## M115
FIRMWARE_NAME:Marlin V1.1.4 (Jun 28 2022 14:16:31) SOURCE_CODE_URL:github.com/MarlinFirmware/Marlin PROTOCOL_VERSION:1.0 MACHINE_TYPE:Ender-3 V2 Neo EXTRUDER_COUNT:1 UUID:cede2a2f-41a2-4748-9b12-c55c62f367ff
Cap:SERIAL_XON_XOFF:0
Cap:BINARY_FILE_TRANSFER:0
Cap:EEPROM:1
Cap:VOLUMETRIC:1
Cap:AUTOREPORT_POS:0
Cap:AUTOREPORT_TEMP:1
Cap:PROGRESS:0
Cap:PRINT_JOB:1
Cap:AUTOLEVEL:1
Cap:RUNOUT:0
Cap:Z_PROBE:1
Cap:LEVELING_DATA:1
Cap:BUILD_PERCENT:0
Cap:SOFTWARE_POWER:0
Cap:TOGGLE_LIGHTS:0
Cap:CASE_LIGHT_BRIGHTNESS:0
Cap:EMERGENCY_PARSER:0
Cap:HOST_ACTION_COMMANDS:0
Cap:PROMPT_SUPPORT:0
Cap:SDCARD:1
Cap:REPEAT:0
Cap:SD_WRITE:1
Cap:AUTOREPORT_SD_STATUS:0
Cap:LONG_FILENAME:1
Cap:THERMAL_PROTECTION:1
Cap:MOTION_MODES:0
Cap:ARCS:1
Cap:BABYSTEPPING:1
Cap:CHAMBER_TEMPERATURE:0
Cap:COOLER_TEMPERATURE:0
Cap:MEATPACK:0
ok

## M119
Reporting endstop status
x_min: open
y_min: open
z_min: TRIGGERED
ok

## M503
*Not response!?
**
In Marlin firmware, the "M503" command doesn't typically return anything directly to the terminal or interface you're using to send commands to the printer. Instead, it reports the current settings stored in the EEPROM (Electrically Erasable Programmable Read-Only Memory) directly to the printer's controller. This information is then usually accessed through the printer's interface, such as an LCD screen or a connected computer running printer control software.

## G28
echo:busy: processing
echo:busy: processing
echo:busy: processing
echo:busy: processing
echo:busy: processing
echo:busy: processing
echo:busy: processing
X:149.20 Y:120.90 Z:11.10 E:0.00 Count X:11936 Y:9672 Z:4440
echo:Bed Leveling ON
echo:Fade Height 3.00
ok

## G29
echo:busy: processing
echo:busy: processing
echo:busy: processing
echo:busy: processing
echo:busy: processing
echo:busy: processing
Bilinear Leveling Grid:
      0      1      2      3
 0 +0.072 -0.012 -0.021 +0.101
 1 +0.075 +0.021 -0.041 +0.046
 2 +0.119 +0.057 +0.006 +0.081
 3 +0.239 +0.154 +0.112 +0.135

Subdivided with CATMULL ROM Leveling Grid:
        0        1        2        3        4        5        6        7        8        9       10       11       12       13       14       15
 0 +0.07250 +0.05439 +0.03508 +0.01636 +0.00007 -0.01200 -0.02074 -0.02735 -0.03049 -0.02882 -0.02100 -0.00492 +0.01853 +0.04619 +0.07490 +0.10150
 1 +0.07234 +0.05589 +0.03854 +0.02164 +0.00656 -0.00533 -0.01530 -0.02425 -0.03030 -0.03154 -0.02608 -0.01175 +0.01019 +0.03648 +0.06386 +0.08906
 2 +0.07151 +0.05688 +0.04167 +0.02675 +0.01300 +0.00130 -0.01001 -0.02152 -0.03073 -0.03514 -0.03224 -0.01978 +0.00058 +0.02545 +0.05144 +0.07518
 3 +0.07101 +0.05812 +0.04496 +0.03194 +0.01946 +0.00796 -0.00464 -0.01861 -0.03085 -0.03830 -0.03786 -0.02721 -0.00840 +0.01507 +0.03971 +0.06202
 4 +0.07184 +0.06039 +0.04892 +0.03746 +0.02603 +0.01467 +0.00101 -0.01496 -0.02971 -0.03968 -0.04132 -0.03225 -0.01483 +0.00736 +0.03074 +0.05174
 5 +0.07500 +0.06444 +0.05403 +0.04355 +0.03278 +0.02150 +0.00718 -0.01005 -0.02637 -0.03796 -0.04100 -0.03310 -0.01680 +0.00430 +0.02660 +0.04650
 6 +0.07993 +0.06963 +0.05952 +0.04931 +0.03873 +0.02749 +0.01302 -0.00449 -0.02118 -0.03322 -0.03676 -0.02942 -0.01377 +0.00663 +0.02822 +0.04744
 7 +0.08596 +0.07543 +0.06505 +0.05459 +0.04385 +0.03260 +0.01840 +0.00137 -0.01478 -0.02635 -0.02967 -0.02240 -0.00700 +0.01303 +0.03423 +0.05311
 8 +0.09394 +0.08284 +0.07179 +0.06072 +0.04957 +0.03828 +0.02455 +0.00845 -0.00662 -0.01723 -0.01995 -0.01256 +0.00267 +0.02239 +0.04322 +0.06181
 9 +0.10467 +0.09284 +0.08094 +0.06907 +0.05736 +0.04593 +0.03273 +0.01767 +0.00383 -0.00569 -0.00782 -0.00043 +0.01444 +0.03357 +0.05378 +0.07184
10 +0.11900 +0.10642 +0.09365 +0.08097 +0.06866 +0.05700 +0.04416 +0.02995 +0.01711 +0.00838 +0.00650 +0.01347 +0.02746 +0.04548 +0.06449 +0.08150
11 +0.13814 +0.12473 +0.11104 +0.09750 +0.08450 +0.07246 +0.05981 +0.04627 +0.03421 +0.02598 +0.02396 +0.02998 +0.04248 +0.05868 +0.07581 +0.09108
12 +0.16153 +0.14712 +0.13234 +0.11774 +0.10390 +0.09137 +0.07887 +0.06603 +0.05477 +0.04701 +0.04469 +0.04946 +0.06004 +0.07393 +0.08866 +0.10173
13 +0.18735 +0.17185 +0.15586 +0.14012 +0.12534 +0.11225 +0.09988 +0.08774 +0.07730 +0.07000 +0.06729 +0.07063 +0.07904 +0.09036 +0.10240 +0.11299
14 +0.21378 +0.19716 +0.17994 +0.16302 +0.14729 +0.13362 +0.12137 +0.10995 +0.10033 +0.09348 +0.09036 +0.09222 +0.09841 +0.10707 +0.11635 +0.12440
15 +0.23900 +0.22130 +0.20291 +0.18487 +0.16822 +0.15400 +0.14188 +0.13117 +0.12237 +0.11598 +0.11250 +0.11297 +0.11706 +0.12320 +0.12987 +0.13550

echo:Settings Stored (698 bytes; crc 49440)
echo:busy: processing
echo:busy: processing
echo:busy: processing
echo:busy: processing
echo:busy: processing
echo:busy: processing
echo:busy: processing
echo:busy: processing
echo:busy: processing
echo:busy: processing
X:149.20 Y:120.90 Z:11.10 E:0.00 Count X:11936 Y:9672 Z:4440
echo:Bed Leveling ON
echo:Fade Height 3.00
X:149.20 Y:120.90 Z:0.00 E:0.00 Count X:11936 Y:9672 Z:4440
ok

