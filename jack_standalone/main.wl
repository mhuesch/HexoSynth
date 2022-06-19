!@import ui;
!@import hx;
!@import node_id;

!:global loaded_tests = $[
    $[
        { std:displayln "STEP 1" @; },
        { std:displayln "STEP 2" },
    ],
    $[
        { std:displayln "XXX STEP 1" },
        { std:displayln "XXX STEP 2" },
    ],
];

!default_style = ui:style[];

!lbl = ui:txt "Test123";

!matrix = hx:get_main_matrix_handle[];

!new_slide_panel = {!(style, panel_layout, child) = @;
    !slide_panel = ui:widget style;
    slide_panel.change_layout panel_layout;
    slide_panel.change_layout ${
        layout_type = :row,
    };

    !close_btn_style = style.clone_set ${
        border = 1,
        border_style = $[:bevel, $f(0, 10, 0, 10)],
    };
    !slide_btn = ui:widget close_btn_style;
    slide_btn.change_layout ${
        width = :pixels => 30,
    };
    !close_btn_text = ui:txt "<";
    slide_btn.set_ctrl :button close_btn_text;
    slide_btn.reg :click {
        if child.is_visible[] {
            child.hide[];
            close_btn_text.set ">";
        } {
            child.show[];
            close_btn_text.set "<";
        };
    };

    slide_panel.add child;
    slide_panel.add slide_btn;

    slide_panel
};

!setup_grid_widget = {!(style, matrix) = @;
    !grid_model = matrix.create_grid_model[];
    !grid = ui:widget style;
    grid.set_ctrl :grid grid_model;

    grid.reg :click {
        std:displayln "GRID CLICK:" @;
        grid_model.set_focus_cell $i(@.1.x, @.1.y);
    };

    grid.reg :drag {
        std:displayln "GRID DRAG:" @;
    };

    grid.change_layout ${
        position_type = :self,
    };

    !panel_style = style.clone_set ${
        border       = 2,
        border_style = $[:rect],
    };

    !add_node_panel_inner = ui:widget panel_style;
    add_node_panel_inner.set_ctrl :rect $n;

    !slide_panel_layout = ${
        top    = :stretch => 1.0,
        width  = :percent => 60,
        height = :pixels  => 200,
    };
    !add_node_panel =
        new_slide_panel
            style
            slide_panel_layout
            add_node_panel_inner;

    !grid_panel = ui:widget style;

    grid_panel.add grid;
    grid_panel.add add_node_panel;

    grid_panel
};

!root = ui:widget default_style;

root.change_layout ${ layout_type = :row };

!grid = setup_grid_widget default_style matrix;
grid.change_layout ${
    height = :stretch => 1.0,
};

!left_panel = ui:widget default_style;
left_panel.change_layout ${
    layout_type = :column,
    width       = :percent => 30,
    min_width   = :pixels  => 300
};
#left_panel.set_ctrl :rect $n;


!param_panel = ui:widget ~ default_style.clone_set ${ };
param_panel.set_ctrl :rect $n;
param_panel.change_layout ${
    height     = :stretch => 2.0,
    min_height = :pixels => 300,
};
!text_panel = ui:widget ~ default_style.clone_set ${ };
text_panel.set_ctrl :rect $n;
text_panel.change_layout ${
    height     = :stretch => 1.0,
    min_height = :pixels => 200,
};

!signal_panel = ui:widget ~ default_style.clone_set ${ };
signal_panel.set_ctrl :rect $n;
signal_panel.change_layout ${
    height     = :stretch => 1.0,
    min_height = :pixels => 300,
};
#param_panel.change_layout ${
#    left = :pixels => 0,
#    right = :pixels => 0,
#};

left_panel.add param_panel;
left_panel.add text_panel;
left_panel.add signal_panel;

root.add left_panel;
root.add grid;

#style.set ${
#    bg_color   = std:v:hex2rgba_f "222",
#    color      = $f(1.0, 1.0, 0.0),
#    font_size  = 24,
#    text_align = :left,
#    pad_left   = 20,
#    border_style = $[:bevel, $f(5.0, 10.0, 20.0, 2.0)],
#};
#
#!btn = ui:widget style;
#btn.set_ctrl :button lbl;
#
#!btn2 = ui:widget style;
#btn2.set_ctrl :button (ui:txt "wurst");
#
#!matrix = hx:get_main_matrix_handle[];
#!freq_s = 440.0;
#!sin_freq = node_id:inp_param $p(:sin, 0) :freq;
#lbl.set ~ sin_freq.format ~ sin_freq.norm freq_s;
#
#btn.reg :click {
#    std:displayln "CLICK!" @;
#    lbl.set ~ sin_freq.format ~ sin_freq.norm freq_s;
#    matrix.set_param sin_freq sin_freq.norm[freq_s];
#    matrix.set_param $p($p(:amp, 0), :att) 1.0;
#    root.remove_child btn2;
#    root.change_layout ${
#        layout_type = :row
#    };
#    .freq_s *= 1.25;
#};
#
#!ent = ui:widget style;
##ent.set_ctrl :entry lbl;
#
##ent.reg :changed {
##    std:displayln "CHANGED" @;
##};
#
#std:displayln "FOO";
#
#iter y 0 => 10 {
#    iter x 0 => 10 {
#        std:displayln ~ matrix.get $i(x, y);
#    };
#};
#
#matrix.set_param $p($p(:amp, 0), :att) 0.0;
#std:displayln ~ node_id:param_list $p(:amp, 0);
#
#!knob_model = matrix.create_hex_knob_dummy_model[];
#!knob = ui:widget style;
#knob.set_ctrl :knob knob_model;
#
#!knob_freq_model = matrix.create_hex_knob_model sin_freq;
#!knob_freq = ui:widget style;
#knob_freq.set_ctrl :knob knob_freq_model;
#
#
#root.add btn;
#root.add ent;
#root.add knob;
#root.add knob_freq;
#root.add btn2;
#root.add grid;

$[root]
