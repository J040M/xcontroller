// This solution might need some more parcing and validation
// Validating the cmd to the printer data to avoid problematic commands

use std::io::{Error, ErrorKind};

pub fn g_command(cmd: &String) -> Result<String, Error> {
    let command = cmd.split_whitespace().next().unwrap();

    match command {
        "G00" | "G01" | "G1" | "G02" | "G2" | "G03" | "G3" | "G04" | "G4" | "G05" | "G5"
        | "G06" | "G6" | "G07" | "G7" | "G08" | "G8" | "G09" | "G9" | "G10" | "G11" | "G12"
        | "G17" | "G18" | "G19" | "G20" | "G21" | "G26" | "G27" | "G28" | "G29" | "G30" | "G31"
        | "G32" | "G33" | "G34" | "G35" | "G38.2" | "G38.3" | "G38.4" | "G38.5" | "G42" | "G53"
        | "G54" | "G55" | "G56" | "G57" | "G58" | "G59" | "G59.1" | "G59.2" | "G59.3" | "G60"
        | "G61" | "G76" | "G80" | "G90" | "G91" | "G92" | "G425" | "M0" | "M1" | "M3" | "M4"
        | "M5" | "M7" | "M8" | "M9" | "M00" | "M01" | "M03" | "M04" | "M05" | "M07" | "M08"
        | "M09" | "M10" | "M11" | "M16" | "M17" | "M18" | "M19" | "M20" | "M21" | "M22" | "M23"
        | "M24" | "M25" | "M26" | "M27" | "M28" | "M29" | "M30" | "M31" | "M32" | "M33" | "M34"
        | "M42" | "M43" | "M48" | "M73" | "M75" | "M76" | "M77" | "M78" | "M80" | "M81" | "M82"
        | "M83" | "M84" |"M85" | "M86" | "M87" | "M92" | "M100" | "M102" | "M104" | "M105" | "M106"
        | "M107" | "M108" | "M109" | "M110" | "M111" | "M112" | "M113" | "M114" | "M115"
        | "M117" | "M118" | "M119" | "M120" | "M121" | "M122" | "M123" | "M125" | "M126"
        | "M127" | "M128" | "M129" | "M140" | "M141" | "M143" | "M145" | "M149" | "M150"
        | "M154" | "M155" | "M163" | "M164" | "M165" | "M166" | "M190" | "M191" | "M192"
        | "M193" | "M200" | "M201" | "M203" | "M204" | "M205" | "M206" | "M207" | "M208"
        | "M209" | "M211" | "M217" | "M218" | "M220" | "M221" | "M226" | "M240" | "M250"
        | "M255" | "M256" | "M260" | "M261" | "M282" | "M290" | "M300" | "M301" | "M302"
        | "M303" | "M304" | "M305" | "M306" | "M350" | "M351" | "M355" | "M360" | "M361"
        | "M362" | "M363" | "M364" | "M380" | "M381" | "M400" | "M401" | "M402" | "M403"
        | "M404" | "M405" | "M406" | "M407" | "M410" | "M412" | "M413" | "M420" | "M421"
        | "M422" | "M423" | "M425" | "M428" | "M430" | "M486" | "M493" | "M500" | "M501"
        | "M502" | "M503" | "M504" | "M510" | "M511" | "M512" | "M524" | "M540" | "M569"
        | "M575" | "M592" | "M593" | "M600" | "M603" | "M605" | "M665" | "M666" | "M672"
        | "M701" | "M702" | "M710" | "M808" | "M810" | "M811" | "M812" | "M813" | "M814"
        | "M815" | "M816" | "M817" | "M818" | "M819" | "M851" | "M852" | "M860" | "M861"
        | "M862" | "M863" | "M864" | "M865" | "M866" | "M867" | "M868" | "M869" | "M871"
        | "M876" | "M900" | "M906" | "M907" | "M908" | "M909" | "M910" | "M911" | "M912"
        | "M913" | "M914" | "M915" | "M916" | "M917" | "M918" | "M919" | "M928" | "M951"
        | "M993" | "M994" | "M995" | "M997" | "M999" | "M7219" | "T00" | "T0" | "T01" | "T1"
        | "T02" | "T2" | "T03" | "T3" | "T04" | "T4" | "T05" | "T5" | "T06" | "T6" | "T07"
        | "T7" | "T08" | "T8" | "T09" | "T9" | "T?" | "Tc" | "Tx" | "S00" | "S0" | "S01" | "S1"
        | "S02" | "S2" | "S03" | "S3" | "S04" | "S4" | "S05" | "S5" | "S06" | "S6" | "S07"
        | "S7" | "S08" | "S8" | "S09" | "S9" | "F00" | "F0" | "F01" | "F1" | "F02" | "F2"
        | "F03" | "F3" | "F04" | "F4" | "F05" | "F5" | "F06" | "F6" | "F07" | "F7" | "F08"
        | "F8" | "F09" | "F9" | "H00" | "H0" | "H01" | "H1" | "H02" | "H2" | "H03" | "H3"
        | "H04" | "H4" | "H05" | "H5" | "H06" | "H6" | "H07" | "H7" | "H08" | "H8" | "H09"
        | "H9" | "D00" | "D0" | "D01" | "D1" | "D02" | "D2" | "D03" | "D3" | "D04" | "D4"
        | "D05" | "D5" | "D06" | "D6" | "D07" | "D7" | "D08" | "D8" | "D09" | "D9" => Ok(cmd.to_string()),
        _ => Err(Error::new(ErrorKind::Other, "Invalid command")),
    }
}
