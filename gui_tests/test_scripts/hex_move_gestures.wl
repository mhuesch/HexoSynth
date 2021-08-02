!@import t = wlambda_lib:test_lib;
!@import hx;
!@import h = wlambda_lib:hex_lib;

!tests = $[];

std:push tests "drag_empty_empty_default_pair" => {
    h:reset10x10[];
    t:matrix_wait {
        h:drag_hex_from_to $i(2, 1) $i(1, 0) :left;
        t:menu_click_text "CV"   :left;
        t:menu_click_text "Ad"   :left;
        t:menu_click_text "CV"   :left;
        t:menu_click_text "TSeq" :left;
    };

    !(a, b) = $[
        hx:get_cell $i(2, 1),
        hx:get_cell $i(1, 0),
    ];
    std:assert_eq a.ports.1 "inp";
    std:assert_eq b.ports.4 "trk1";
};

std:push tests "drag_empty_empty_default_pair_rev" => {
    h:reset10x10[];
    t:matrix_wait {
        h:drag_hex_from_to $i(1, 0) $i(2, 1) :left;
        t:menu_click_text "CV"   :left;
        t:menu_click_text "TSeq" :left;
        t:menu_click_text "CV"   :left;
        t:menu_click_text "Ad"   :left;
    };

    !(a, b) = $[
        hx:get_cell $i(2, 1),
        hx:get_cell $i(1, 0),
    ];
    std:assert_eq a.ports.1 "inp";
    std:assert_eq b.ports.4 "trk1";
};

std:push tests "drag_empty_empty_set_io" => {
    h:reset10x10[];
    t:matrix_wait {
        h:drag_hex_from_to $i(1, 0) $i(2, 1) :right;
        t:menu_click_text "CV"   :left;
        t:menu_click_text "TSeq" :left;
        t:menu_click_text "CV"   :left;
        t:menu_click_text "Ad"   :left;
        t:menu_click_text "trk3" :left;
        t:menu_click_text "trig" :left;
    };

    !(a, b) = $[
        hx:get_cell $i(2, 1),
        hx:get_cell $i(1, 0),
    ];

    std:assert_eq a.ports.1 "trig";
    std:assert_eq b.ports.4 "trk3";
};

std:push tests "drag_empty_empty_set_io_rev" => {
    h:reset10x10[];
    t:matrix_wait {
        h:drag_hex_from_to $i(2, 1) $i(1, 0) :right;
        t:menu_click_text "CV"   :left;
        t:menu_click_text "Ad"   :left;
        t:menu_click_text "CV"   :left;
        t:menu_click_text "TSeq" :left;
        t:menu_click_text "trig" :left;
        t:menu_click_text "trk3" :left;
    };

    !(a, b) = $[
        hx:get_cell $i(2, 1),
        hx:get_cell $i(1, 0),
    ];

    std:assert_eq a.ports.1 "trig";
    std:assert_eq b.ports.4 "trk3";
};

!setup_sin_sin_cluster = {!(a_pos, b_pos) = @;
    h:reset10x10[];

    t:matrix_wait {
        hx:set_cell a_pos ${
            node_id = "sin" => 0,
            ports   = $[$n, $n, $n, $n, "sig"],
        };
    };

    t:matrix_wait {
        hx:set_cell b_pos ${
            node_id = "sin" => 1,
            ports   = $[$n, "det", $n, $n, $n, $n],
        };
    };

};

std:push tests "drag_cluster" => {
    setup_sin_sin_cluster $i(1, 1) $i(2, 2);

    t:matrix_wait {
        h:drag_hex_from_to $i(2, 2) $i(2, 1) :left;
    };

    !(a, b) = $[
        hx:get_cell $i(1, 0),
        hx:get_cell $i(2, 1),
    ];
    std:assert_eq a.node_id.0 "Sin";
    std:assert_eq b.node_id.0 "Sin";

    std:assert_eq b.ports.1 "det";
    std:assert_eq a.ports.4 "sig";

    hx:query_state[];
};

std:push tests "drag_cluster_err" => {
    setup_sin_sin_cluster $i(1, 1) $i(2, 2);

    h:drag_hex_from_to $i(2, 2) $i(2, 0) :left;

    hx:query_state[];

    !id = hx:id_by_text_contains "out of Range";
    std:assert_eq id.0.0.1 "DBGID_TEXT_HEADER";
};

tests
