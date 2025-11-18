// editorconfig-checker-disable
compile_error!(
    r#"
failed to parse `fragment.dbc`

Caused by:
      --> 40:51
   |
40 |  SG_ EngineSpeed : 24|16@1+ (0.125,0) [0|8031.875]␊
   |                                                   ^---
   |
   = expected unit
"#
);
