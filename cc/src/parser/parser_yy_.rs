#![allow(clippy::useless_conversion)]
use crate::types::ast::ast_nodes::*;
use crate::types::ast::decl_info::*;
use crate::types::ast::func_info::*;
use crate::types::ast::initializer::*;
use crate::types::ast::parser_node::*;
use crate::types::ast::type_info::*;
use LRAction::*;


/// vector deconstruct into vars
macro_rules! destruct_vec {
    ($vec:expr, $($var:ident),*) => {
        let [$($var),*] = $vec.try_into().expect("destruct failed");
    };
}

pub const INIT_STATE: usize = 0;
pub const END_SYMBOL: usize = 0;

/// LR Table Cell Type
#[derive(Debug, Clone)]
pub enum LRAction {
    Reduce(usize),
    Shift(usize),
    Accept(usize),
    Error
}


/// action matrix -> base next check
static ACTION_BASE: [Option<usize>; 348] = [
    Some(7307), Some(853), Some(6273), Some(6710), Some(6662), Some(6642), Some(6558), Some(6377), Some(6225), 
    Some(6205), Some(6157), Some(6109), Some(5937), Some(5889), Some(5837), Some(5816), Some(5768), Some(5748), 
    Some(5668), Some(487), Some(482), Some(429), Some(7288), Some(7234), Some(7180), Some(178), Some(6293), 
    Some(5483), Some(344), Some(5383), Some(1324), Some(353), Some(71), Some(841), Some(1064), Some(359), 
    Some(4953), Some(520), Some(7154), Some(582), Some(547), Some(545), Some(273), Some(1387), Some(893), 
    Some(80), Some(10), Some(4873), Some(496), Some(493), Some(1260), Some(830), Some(181), Some(5403), 
    Some(5357), Some(140), Some(1001), Some(351), Some(2380), Some(3395), Some(189), Some(2965), Some(7133), 
    Some(336), Some(74), Some(557), Some(4808), Some(5899), Some(414), Some(536), Some(421), Some(544), 
    Some(8078), Some(8056), Some(4724), Some(8045), Some(8034), Some(8021), Some(8010), Some(4187), Some(4160), 
    Some(4119), Some(4092), Some(3295), Some(7986), Some(7958), Some(541), Some(3843), Some(3816), Some(3268), 
    Some(3775), Some(4916), Some(7934), Some(7093), Some(7052), Some(5810), Some(7642), Some(7835), Some(8081), 
    Some(8288), Some(490), Some(1043), Some(2031), Some(50), Some(156), Some(160), Some(7086), Some(7448), 
    Some(970), Some(924), Some(134), Some(2186), Some(60), Some(4228), Some(528), Some(518), Some(516), 
    Some(505), Some(2889), Some(455), Some(236), Some(407), Some(384), Some(5096), Some(2874), Some(2680), 
    Some(2095), Some(2036), Some(2589), Some(2530), Some(1977), Some(1886), Some(1692), Some(1601), Some(109), 
    Some(492), Some(5335), Some(538), Some(299), Some(397), Some(537), Some(363), Some(506), Some(500), 
    Some(309), Some(4667), Some(86), Some(154), Some(5309), Some(38), Some(413), Some(200), Some(418), 
    Some(417), Some(4626), Some(4599), Some(4558), Some(3019), Some(3085), Some(21), Some(12), Some(3748), 
    Some(3707), Some(7923), Some(7892), Some(7774), Some(7685), Some(6605), Some(4219), Some(3882), Some(3807), 
    Some(3739), Some(3470), Some(3327), Some(4531), Some(1675), Some(1549), Some(193), Some(5289), Some(476), 
    Some(420), Some(401), Some(166), Some(178), Some(27), Some(4292), Some(171), Some(1542), Some(1483), 
    Some(1392), Some(173), Some(0), Some(2471), Some(1198), Some(412), Some(72), Some(793), Some(411), 
    Some(350), Some(5218), Some(7), Some(5024), Some(766), Some(5002), Some(353), Some(26), Some(304), 
    Some(181), Some(3680), Some(1988), Some(157), Some(162), Some(146), Some(343), Some(301), Some(3431), 
    Some(13), Some(3404), Some(73), Some(7025), Some(6998), Some(6702), Some(6566), Some(6265), Some(2975), 
    Some(3), Some(7597), Some(7584), Some(7574), Some(4995), Some(7738), Some(7715), Some(5345), Some(5440), 
    Some(114), Some(549), Some(102), Some(1537), Some(169), Some(430), Some(1107), Some(1048), Some(287), 
    Some(239), Some(213), Some(130), Some(106), Some(12), Some(989), Some(898), Some(14), Some(6730), 
    Some(165), Some(185), Some(106), Some(150), Some(6953), Some(4504), Some(31), Some(24), Some(4255), 
    Some(3363), Some(3336), Some(168), Some(110), Some(704), Some(101), Some(99), Some(74), Some(17), 
    Some(6489), Some(51), Some(23), Some(613), Some(554), Some(495), Some(33), Some(45), Some(155), Some(149), 
    Some(97), Some(47), Some(6), Some(17), Some(2482), Some(45), Some(43), Some(404), Some(210), Some(8), 
    Some(119), None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, 
    None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, 
    None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, 
    None, None, None, None, None, None, None, None, 
];

static ACTION_NEXT: [LRAction; 8564] = [
    Reduce(117), Error, Error, Error, Error, Error, Error, Error, Error, Error, Error, Error, Error, 
    Error, Error, Error, Error, Error, Error, Error, Error, Error, Error, Error, Error, Error, Error, 
    Error, Error, Error, Error, Error, Error, Reduce(117), Error, Error, Error, Error, Reduce(117), Error, 
    Reduce(117), Reduce(179), Reduce(117), Reduce(117), Reduce(179), Reduce(117), Shift(194), Reduce(179), 
    Shift(195), Shift(346), Reduce(19), Reduce(19), Reduce(19), Reduce(131), Reduce(19), Reduce(215), 
    Shift(234), Shift(234), Reduce(215), Reduce(117), Reduce(116), Reduce(179), Reduce(179), Reduce(179), 
    Reduce(200), Reduce(179), Reduce(179), Reduce(200), Reduce(106), Reduce(19), Reduce(62), Reduce(131), 
    Reduce(215), Reduce(215), Shift(339), Reduce(104), Shift(344), Shift(234), Reduce(66), Reduce(91), 
    Shift(1), Reduce(200), Reduce(200), Reduce(97), Reduce(97), Reduce(99), Reduce(99), Reduce(97), Shift(341), 
    Reduce(99), Reduce(104), Reduce(199), Reduce(156), Reduce(116), Reduce(199), Reduce(156), Reduce(179), 
    Reduce(179), Reduce(116), Shift(342), Reduce(116), Reduce(19), Reduce(116), Reduce(116), Shift(340), 
    Reduce(116), Shift(310), Reduce(215), Reduce(199), Reduce(199), Shift(335), Reduce(67), Shift(297), 
    Shift(208), Reduce(202), Shift(334), Reduce(200), Reduce(202), Reduce(15), Reduce(116), Reduce(18), 
    Reduce(18), Reduce(18), Reduce(117), Reduce(18), Reduce(117), Reduce(117), Reduce(179), Reduce(179), 
    Reduce(66), Reduce(216), Reduce(202), Reduce(202), Reduce(15), Reduce(97), Shift(64), Reduce(99), 
    Reduce(96), Reduce(96), Reduce(18), Shift(333), Reduce(96), Reduce(101), Reduce(199), Reduce(216), 
    Reduce(216), Shift(234), Reduce(86), Reduce(200), Reduce(106), Reduce(214), Reduce(62), Reduce(129), 
    Reduce(214), Reduce(58), Reduce(193), Reduce(104), Reduce(129), Reduce(193), Reduce(129), Shift(311), 
    Reduce(129), Reduce(129), Shift(298), Reduce(129), Shift(319), Reduce(202), Reduce(214), Reduce(214), 
    Reduce(58), Shift(318), Reduce(18), Reduce(193), Reduce(193), Reduce(66), Reduce(199), Shift(1), 
    Reduce(193), Reduce(129), Reduce(216), Reduce(73), Reduce(73), Reduce(73), Reduce(116), Reduce(73), 
    Reduce(116), Reduce(116), Shift(308), Reduce(96), Reduce(98), Reduce(98), Reduce(88), Shift(215), 
    Reduce(98), Reduce(88), Reduce(95), Reduce(95), Reduce(201), Reduce(202), Reduce(95), Reduce(201), 
    Reduce(103), Reduce(214), Shift(307), Reduce(102), Shift(324), Reduce(94), Reduce(193), Shift(205), 
    Reduce(94), Reduce(56), Reduce(216), Reduce(55), Reduce(57), Reduce(201), Reduce(201), Reduce(103), 
    Shift(234), Reduce(66), Reduce(102), Shift(1), Reduce(66), Reduce(217), Shift(1), Shift(284), Reduce(56), 
    Reduce(84), Reduce(55), Reduce(57), Reduce(84), Shift(294), Reduce(73), Shift(295), Reduce(51), Reduce(51), 
    Reduce(51), Shift(286), Reduce(11), Reduce(193), Reduce(193), Reduce(98), Shift(251), Reduce(129), 
    Reduce(128), Reduce(129), Reduce(129), Reduce(95), Reduce(80), Reduce(128), Reduce(201), Reduce(128), 
    Reduce(51), Reduce(128), Reduce(128), Shift(317), Reduce(128), Shift(325), Shift(234), Reduce(117), 
    Reduce(117), Reduce(117), Reduce(117), Reduce(117), Reduce(117), Reduce(117), Shift(300), Reduce(117), 
    Reduce(117), Reduce(19), Reduce(128), Shift(260), Reduce(179), Reduce(179), Reduce(179), Reduce(179), 
    Reduce(179), Reduce(179), Reduce(179), Reduce(179), Shift(258), Shift(316), Reduce(201), Reduce(103), 
    Shift(234), Reduce(51), Reduce(102), Reduce(117), Reduce(117), Reduce(117), Reduce(117), Reduce(117), 
    Reduce(117), Reduce(117), Reduce(117), Reduce(117), Reduce(117), Reduce(117), Reduce(117), Reduce(117), 
    Reduce(117), Reduce(117), Reduce(117), Reduce(117), Reduce(117), Reduce(117), Reduce(117), Reduce(117), 
    Reduce(117), Reduce(117), Reduce(117), Reduce(117), Reduce(117), Reduce(117), Reduce(117), Reduce(117), 
    Reduce(117), Reduce(117), Reduce(15), Reduce(116), Reduce(116), Reduce(116), Reduce(116), Reduce(116), 
    Reduce(116), Reduce(116), Shift(209), Reduce(116), Reduce(116), Shift(315), Reduce(67), Shift(57), 
    Shift(234), Reduce(15), Reduce(128), Shift(64), Reduce(128), Reduce(128), Shift(291), Reduce(18), 
    Reduce(66), Reduce(91), Shift(1), Reduce(155), Reduce(91), Shift(303), Reduce(155), Reduce(116), 
    Reduce(116), Reduce(116), Reduce(116), Reduce(116), Reduce(116), Reduce(116), Reduce(116), Reduce(116), 
    Reduce(116), Reduce(116), Reduce(116), Reduce(116), Reduce(116), Reduce(116), Reduce(116), Reduce(116), 
    Reduce(116), Reduce(116), Reduce(116), Reduce(116), Reduce(116), Reduce(116), Reduce(116), Reduce(116), 
    Reduce(116), Reduce(116), Reduce(116), Reduce(116), Reduce(116), Reduce(116), Reduce(129), Reduce(129), 
    Reduce(129), Reduce(129), Reduce(129), Reduce(129), Reduce(129), Reduce(154), Reduce(129), Reduce(129), 
    Shift(309), Reduce(193), Reduce(193), Reduce(66), Reduce(92), Reduce(66), Shift(56), Reduce(92), 
    Shift(298), Reduce(6), Reduce(64), Reduce(73), Reduce(69), Reduce(69), Shift(1), Shift(245), Reduce(69), 
    Shift(243), Reduce(129), Reduce(129), Reduce(129), Reduce(129), Reduce(129), Reduce(129), Reduce(129), 
    Reduce(129), Reduce(129), Reduce(129), Reduce(129), Reduce(129), Reduce(129), Reduce(129), Reduce(129), 
    Reduce(129), Reduce(129), Reduce(129), Reduce(129), Reduce(129), Reduce(129), Reduce(129), Reduce(129), 
    Reduce(129), Reduce(129), Reduce(129), Reduce(129), Reduce(129), Reduce(129), Reduce(129), Reduce(129), 
    Reduce(66), Reduce(125), Shift(241), Reduce(66), Reduce(73), Reduce(73), Reduce(125), Shift(229), 
    Reduce(125), Reduce(54), Reduce(125), Reduce(125), Shift(2), Reduce(125), Reduce(69), Reduce(51), 
    Reduce(90), Reduce(89), Shift(250), Reduce(90), Reduce(89), Shift(234), Reduce(63), Reduce(9), Reduce(54), 
    Shift(252), Reduce(105), Reduce(125), Shift(282), Reduce(61), Shift(228), Reduce(45), Reduce(128), 
    Reduce(128), Reduce(128), Reduce(128), Reduce(128), Reduce(128), Reduce(128), Shift(158), Reduce(128), 
    Reduce(128), Reduce(64), Shift(283), Shift(8), Shift(9), Shift(10), Shift(11), Shift(12), Shift(13), 
    Shift(14), Shift(15), Shift(16), Shift(17), Shift(18), Shift(19), Shift(20), Shift(21), Shift(227), 
    Shift(226), Reduce(128), Reduce(128), Reduce(128), Reduce(128), Reduce(128), Reduce(128), Reduce(128), 
    Reduce(128), Reduce(128), Reduce(128), Reduce(128), Reduce(128), Reduce(128), Reduce(128), Reduce(128), 
    Reduce(128), Reduce(128), Reduce(128), Reduce(128), Reduce(128), Reduce(128), Reduce(128), Reduce(128), 
    Reduce(128), Reduce(128), Reduce(128), Reduce(128), Reduce(128), Reduce(128), Reduce(128), Reduce(128), 
    Reduce(125), Reduce(127), Reduce(125), Reduce(125), Reduce(194), Shift(2), Reduce(127), Reduce(194), 
    Reduce(127), Shift(234), Reduce(127), Reduce(127), Reduce(63), Reduce(127), Reduce(87), Shift(253), 
    Reduce(105), Reduce(87), Shift(224), Reduce(61), Reduce(85), Reduce(194), Reduce(194), Shift(244), 
    Shift(235), Reduce(45), Reduce(194), Reduce(127), Reduce(49), Shift(223), Reduce(66), Shift(222), 
    Shift(3), Shift(4), Shift(5), Shift(6), Shift(7), Shift(8), Shift(9), Shift(10), Shift(11), Shift(12), 
    Shift(13), Shift(14), Shift(15), Shift(16), Shift(17), Shift(18), Shift(19), Shift(20), Shift(21), 
    Reduce(82), Reduce(83), Shift(159), Shift(242), Reduce(83), Reduce(194), Shift(205), Reduce(16), 
    Shift(221), Reduce(126), Reduce(14), Reduce(13), Reduce(195), Shift(63), Reduce(126), Reduce(195), 
    Reduce(126), Reduce(9), Reduce(126), Reduce(126), Shift(146), Reduce(126), Reduce(16), Reduce(48), 
    Shift(50), Reduce(14), Reduce(13), Reduce(44), Reduce(12), Reduce(195), Reduce(195), Shift(74), Reduce(43), 
    Shift(57), Reduce(195), Reduce(126), Reduce(194), Reduce(194), Shift(67), Reduce(69), Reduce(127), 
    Shift(66), Reduce(127), Reduce(127), Reduce(9), Reduce(9), Reduce(9), Reduce(9), Reduce(9), Reduce(9), 
    Reduce(9), Reduce(9), Reduce(9), Reduce(9), Reduce(9), Reduce(9), Reduce(9), Reduce(9), Reduce(9), 
    Reduce(9), Reduce(9), Reduce(9), Reduce(9), Shift(62), Reduce(195), Shift(61), Error, Error, Reduce(124), 
    Error, Error, Error, Error, Reduce(124), Error, Reduce(124), Error, Reduce(124), Reduce(124), Error, 
    Reduce(124), Shift(17), Shift(18), Shift(160), Reduce(125), Reduce(125), Reduce(125), Reduce(125), 
    Reduce(125), Reduce(125), Reduce(125), Error, Reduce(125), Reduce(125), Reduce(124), Shift(206), 
    Reduce(195), Error, Error, Reduce(126), Error, Reduce(126), Reduce(126), Error, Error, Error, Error, 
    Error, Error, Shift(39), Error, Reduce(49), Reduce(125), Reduce(125), Reduce(125), Reduce(125), Reduce(125), 
    Reduce(125), Reduce(125), Reduce(125), Reduce(125), Reduce(125), Reduce(125), Reduce(125), Reduce(125), 
    Reduce(125), Reduce(125), Reduce(125), Reduce(125), Reduce(125), Reduce(125), Reduce(125), Reduce(125), 
    Reduce(125), Reduce(125), Reduce(125), Reduce(125), Reduce(125), Reduce(125), Reduce(125), Reduce(125), 
    Reduce(125), Reduce(125), Reduce(49), Reduce(49), Reduce(49), Reduce(49), Reduce(49), Reduce(49), 
    Reduce(49), Reduce(49), Reduce(49), Reduce(49), Reduce(49), Reduce(49), Reduce(49), Reduce(49), Reduce(48), 
    Reduce(124), Reduce(114), Reduce(124), Reduce(124), Reduce(44), Error, Reduce(114), Error, Reduce(114), 
    Reduce(43), Reduce(114), Reduce(114), Error, Reduce(114), Error, Error, Error, Reduce(127), Reduce(127), 
    Reduce(127), Reduce(127), Reduce(127), Reduce(127), Reduce(127), Error, Reduce(127), Reduce(127), 
    Reduce(114), Reduce(194), Reduce(194), Error, Reduce(48), Reduce(48), Reduce(48), Reduce(48), Reduce(48), 
    Reduce(48), Reduce(48), Reduce(48), Reduce(48), Reduce(48), Reduce(48), Reduce(48), Reduce(48), Reduce(48), 
    Reduce(127), Reduce(127), Reduce(127), Reduce(127), Reduce(127), Reduce(127), Reduce(127), Reduce(127), 
    Reduce(127), Reduce(127), Reduce(127), Reduce(127), Reduce(127), Reduce(127), Reduce(127), Reduce(127), 
    Reduce(127), Reduce(127), Reduce(127), Reduce(127), Reduce(127), Reduce(127), Reduce(127), Reduce(127), 
    Reduce(127), Reduce(127), Reduce(127), Reduce(127), Reduce(127), Reduce(127), Reduce(127), Reduce(126), 
    Reduce(126), Reduce(126), Reduce(126), Reduce(126), Reduce(126), Reduce(126), Error, Reduce(126), 
    Reduce(126), Error, Reduce(195), Reduce(195), Error, Error, Reduce(114), Error, Reduce(114), Reduce(114), 
    Error, Error, Reduce(67), Reduce(93), Error, Error, Reduce(93), Error, Error, Reduce(126), Reduce(126), 
    Reduce(126), Reduce(126), Reduce(126), Reduce(126), Reduce(126), Reduce(126), Reduce(126), Reduce(126), 
    Reduce(126), Reduce(126), Reduce(126), Reduce(126), Reduce(126), Reduce(126), Reduce(126), Reduce(126), 
    Reduce(126), Reduce(126), Reduce(126), Reduce(126), Reduce(126), Reduce(126), Reduce(126), Reduce(126), 
    Reduce(126), Reduce(126), Reduce(126), Reduce(126), Reduce(126), Reduce(124), Reduce(124), Reduce(124), 
    Reduce(124), Reduce(124), Reduce(124), Reduce(124), Error, Reduce(124), Reduce(124), Reduce(72), 
    Reduce(72), Reduce(72), Reduce(67), Reduce(72), Error, Error, Error, Error, Error, Error, Error, 
    Reduce(68), Reduce(68), Shift(1), Error, Reduce(68), Error, Reduce(124), Reduce(124), Reduce(124), 
    Reduce(124), Reduce(124), Reduce(124), Reduce(124), Reduce(124), Reduce(124), Reduce(124), Reduce(124), 
    Reduce(124), Reduce(124), Reduce(124), Reduce(124), Reduce(124), Reduce(124), Reduce(124), Reduce(124), 
    Reduce(124), Reduce(124), Reduce(124), Shift(338), Reduce(124), Reduce(124), Reduce(124), Reduce(124), 
    Reduce(124), Reduce(124), Reduce(124), Reduce(124), Error, Reduce(136), Reduce(72), Reduce(17), Reduce(17), 
    Reduce(17), Reduce(136), Reduce(17), Reduce(136), Error, Reduce(136), Reduce(136), Error, Reduce(136), 
    Reduce(68), Error, Error, Error, Error, Error, Error, Error, Reduce(17), Reduce(8), Error, Error, 
    Error, Reduce(136), Error, Error, Error, Error, Reduce(114), Reduce(114), Reduce(114), Reduce(114), 
    Reduce(114), Reduce(114), Reduce(114), Error, Reduce(114), Reduce(114), Error, Error, Error, Error, 
    Error, Error, Error, Error, Error, Error, Error, Error, Reduce(17), Error, Error, Error, Error, Error, 
    Reduce(114), Reduce(114), Reduce(114), Reduce(114), Reduce(114), Reduce(114), Reduce(114), Reduce(114), 
    Reduce(114), Reduce(114), Reduce(114), Reduce(114), Reduce(114), Reduce(114), Reduce(114), Reduce(114), 
    Reduce(114), Reduce(114), Reduce(114), Reduce(114), Reduce(114), Reduce(114), Reduce(114), Reduce(114), 
    Reduce(114), Reduce(114), Reduce(114), Reduce(114), Reduce(114), Reduce(114), Reduce(114), Reduce(136), 
    Reduce(132), Reduce(136), Reduce(136), Shift(2), Error, Reduce(132), Error, Reduce(132), Error, Reduce(132), 
    Reduce(132), Error, Reduce(132), Error, Error, Error, Error, Error, Error, Reduce(71), Reduce(71), 
    Error, Error, Reduce(71), Error, Error, Reduce(132), Reduce(47), Error, Reduce(67), Shift(3), Shift(4), 
    Shift(5), Shift(6), Shift(7), Shift(8), Shift(9), Shift(10), Shift(11), Shift(12), Shift(13), Shift(14), 
    Shift(15), Shift(16), Shift(17), Shift(18), Shift(19), Shift(20), Shift(21), Error, Error, Error, 
    Error, Error, Error, Error, Error, Error, Error, Reduce(115), Error, Shift(301), Reduce(196), Error, 
    Reduce(115), Reduce(196), Reduce(115), Reduce(8), Reduce(115), Reduce(115), Reduce(71), Reduce(115), 
    Error, Shift(213), Error, Error, Error, Reduce(72), Error, Reduce(196), Reduce(196), Error, Reduce(70), 
    Reduce(70), Reduce(196), Reduce(115), Reduce(70), Error, Error, Reduce(68), Reduce(132), Error, Reduce(132), 
    Reduce(132), Reduce(8), Reduce(8), Reduce(8), Reduce(8), Reduce(8), Reduce(8), Reduce(8), Reduce(8), 
    Reduce(8), Reduce(8), Reduce(8), Reduce(8), Reduce(8), Reduce(8), Reduce(8), Reduce(8), Reduce(8), 
    Reduce(8), Reduce(8), Error, Reduce(196), Error, Error, Error, Reduce(113), Reduce(72), Reduce(72), 
    Error, Error, Reduce(113), Error, Reduce(113), Error, Reduce(113), Reduce(113), Reduce(17), Reduce(113), 
    Shift(17), Shift(18), Reduce(70), Reduce(136), Reduce(136), Reduce(136), Reduce(136), Reduce(136), 
    Reduce(136), Reduce(136), Error, Reduce(136), Reduce(136), Reduce(113), Shift(206), Reduce(196), 
    Error, Error, Reduce(115), Error, Reduce(115), Reduce(115), Error, Error, Error, Error, Error, Error, 
    Error, Error, Reduce(47), Reduce(136), Reduce(136), Reduce(136), Reduce(136), Reduce(136), Reduce(136), 
    Reduce(136), Reduce(136), Reduce(136), Reduce(136), Reduce(136), Reduce(136), Reduce(136), Reduce(136), 
    Reduce(136), Reduce(136), Reduce(136), Reduce(136), Reduce(136), Reduce(136), Reduce(136), Reduce(136), 
    Reduce(136), Reduce(136), Reduce(136), Reduce(136), Reduce(136), Reduce(136), Reduce(136), Reduce(136), 
    Reduce(136), Reduce(47), Reduce(47), Reduce(47), Reduce(47), Reduce(47), Reduce(47), Reduce(47), 
    Reduce(47), Reduce(47), Reduce(47), Reduce(47), Reduce(47), Reduce(47), Reduce(47), Shift(2), Reduce(113), 
    Reduce(123), Reduce(113), Reduce(113), Error, Error, Reduce(123), Error, Reduce(123), Error, Reduce(123), 
    Reduce(123), Error, Reduce(123), Error, Error, Error, Reduce(132), Reduce(132), Reduce(132), Reduce(132), 
    Reduce(132), Reduce(132), Reduce(132), Error, Reduce(132), Reduce(132), Reduce(123), Error, Reduce(71), 
    Error, Shift(8), Shift(9), Shift(10), Shift(11), Shift(12), Shift(13), Shift(14), Shift(15), Shift(16), 
    Shift(17), Shift(18), Shift(19), Shift(20), Shift(21), Reduce(132), Reduce(132), Reduce(132), Reduce(132), 
    Reduce(132), Reduce(132), Reduce(132), Reduce(132), Reduce(132), Reduce(132), Reduce(132), Reduce(132), 
    Reduce(132), Reduce(132), Reduce(132), Reduce(132), Reduce(132), Reduce(132), Reduce(132), Reduce(132), 
    Reduce(132), Reduce(132), Reduce(132), Reduce(132), Reduce(132), Reduce(132), Reduce(132), Reduce(132), 
    Reduce(132), Reduce(132), Reduce(132), Reduce(115), Reduce(115), Reduce(115), Reduce(115), Reduce(115), 
    Reduce(115), Reduce(115), Error, Reduce(115), Reduce(115), Error, Reduce(196), Reduce(196), Error, 
    Error, Reduce(123), Reduce(70), Reduce(123), Reduce(123), Error, Error, Error, Error, Error, Error, 
    Error, Error, Error, Reduce(115), Reduce(115), Reduce(115), Reduce(115), Reduce(115), Reduce(115), 
    Reduce(115), Reduce(115), Reduce(115), Reduce(115), Reduce(115), Reduce(115), Reduce(115), Reduce(115), 
    Reduce(115), Reduce(115), Reduce(115), Reduce(115), Reduce(115), Reduce(115), Reduce(115), Reduce(115), 
    Reduce(115), Reduce(115), Reduce(115), Reduce(115), Reduce(115), Reduce(115), Reduce(115), Reduce(115), 
    Reduce(115), Reduce(113), Reduce(113), Reduce(113), Reduce(113), Reduce(113), Reduce(113), Reduce(113), 
    Error, Reduce(113), Reduce(113), Error, Error, Error, Error, Error, Error, Error, Error, Reduce(7), 
    Error, Error, Error, Error, Error, Error, Error, Error, Error, Reduce(113), Reduce(113), Reduce(113), 
    Reduce(113), Reduce(113), Reduce(113), Reduce(113), Reduce(113), Reduce(113), Reduce(113), Reduce(113), 
    Reduce(113), Reduce(113), Reduce(113), Reduce(113), Reduce(113), Reduce(113), Reduce(113), Reduce(113), 
    Reduce(113), Reduce(113), Reduce(113), Reduce(113), Reduce(113), Reduce(113), Reduce(113), Reduce(113), 
    Reduce(113), Reduce(113), Reduce(113), Reduce(113), Error, Reduce(135), Error, Reduce(21), Reduce(21), 
    Reduce(21), Reduce(135), Reduce(21), Reduce(135), Error, Reduce(135), Reduce(135), Error, Reduce(135), 
    Error, Error, Error, Error, Error, Error, Error, Error, Reduce(21), Reduce(6), Error, Error, Error, 
    Reduce(135), Error, Error, Error, Error, Reduce(123), Reduce(123), Reduce(123), Reduce(123), Reduce(123), 
    Reduce(123), Reduce(123), Error, Reduce(123), Reduce(123), Error, Error, Error, Error, Error, Error, 
    Error, Error, Error, Error, Error, Error, Reduce(21), Error, Error, Error, Error, Error, Reduce(123), 
    Reduce(123), Reduce(123), Reduce(123), Reduce(123), Reduce(123), Reduce(123), Reduce(123), Reduce(123), 
    Reduce(123), Reduce(123), Reduce(123), Reduce(123), Reduce(123), Reduce(123), Reduce(123), Reduce(123), 
    Reduce(123), Reduce(123), Reduce(123), Reduce(123), Reduce(123), Reduce(123), Reduce(123), Reduce(123), 
    Reduce(123), Reduce(123), Reduce(123), Reduce(123), Reduce(123), Reduce(123), Reduce(135), Reduce(134), 
    Reduce(135), Reduce(135), Shift(2), Error, Reduce(134), Error, Reduce(134), Error, Reduce(134), Reduce(134), 
    Error, Reduce(134), Error, Error, Error, Error, Error, Error, Error, Error, Error, Error, Error, 
    Error, Error, Reduce(134), Error, Error, Error, Shift(3), Shift(4), Shift(5), Shift(6), Shift(7), 
    Shift(8), Shift(9), Shift(10), Shift(11), Shift(12), Shift(13), Shift(14), Shift(15), Shift(16), 
    Shift(17), Shift(18), Shift(19), Shift(20), Shift(21), Error, Error, Error, Error, Error, Error, 
    Error, Error, Error, Error, Reduce(133), Error, Error, Reduce(197), Error, Reduce(133), Reduce(197), 
    Reduce(133), Shift(2), Reduce(133), Reduce(133), Error, Reduce(133), Error, Reduce(50), Reduce(50), 
    Reduce(50), Error, Error, Error, Reduce(197), Reduce(197), Error, Error, Error, Reduce(197), Reduce(133), 
    Error, Error, Error, Error, Reduce(134), Reduce(50), Reduce(134), Reduce(134), Shift(3), Shift(4), 
    Shift(5), Shift(6), Shift(7), Shift(8), Shift(9), Shift(10), Shift(11), Shift(12), Shift(13), Shift(14), 
    Shift(15), Shift(16), Shift(17), Shift(18), Shift(19), Shift(20), Shift(21), Error, Reduce(197), 
    Error, Error, Error, Reduce(112), Error, Error, Error, Error, Reduce(112), Reduce(50), Reduce(112), 
    Error, Reduce(112), Reduce(112), Reduce(21), Reduce(112), Error, Error, Error, Reduce(135), Reduce(135), 
    Reduce(135), Reduce(135), Reduce(135), Reduce(135), Reduce(135), Error, Reduce(135), Reduce(135), 
    Reduce(112), Error, Reduce(197), Error, Error, Reduce(133), Error, Reduce(133), Reduce(133), Error, 
    Error, Error, Error, Error, Error, Error, Error, Error, Reduce(135), Reduce(135), Reduce(135), Reduce(135), 
    Reduce(135), Reduce(135), Reduce(135), Reduce(135), Reduce(135), Reduce(135), Reduce(135), Reduce(135), 
    Reduce(135), Reduce(135), Reduce(135), Reduce(135), Reduce(135), Reduce(135), Reduce(135), Reduce(135), 
    Reduce(135), Reduce(135), Reduce(135), Reduce(135), Reduce(135), Reduce(135), Reduce(135), Reduce(135), 
    Reduce(135), Reduce(135), Reduce(135), Error, Error, Error, Error, Error, Error, Reduce(53), Reduce(53), 
    Reduce(53), Error, Error, Error, Error, Error, Error, Reduce(112), Reduce(111), Reduce(112), Reduce(112), 
    Error, Error, Reduce(111), Error, Reduce(111), Reduce(53), Reduce(111), Reduce(111), Error, Reduce(111), 
    Error, Error, Error, Reduce(134), Reduce(134), Reduce(134), Reduce(134), Reduce(134), Reduce(134), 
    Reduce(134), Error, Reduce(134), Reduce(134), Reduce(111), Error, Error, Error, Error, Error, Error, 
    Error, Error, Error, Error, Error, Error, Error, Error, Reduce(53), Error, Error, Reduce(134), Reduce(134), 
    Reduce(134), Reduce(134), Reduce(134), Reduce(134), Reduce(134), Reduce(134), Reduce(134), Reduce(134), 
    Reduce(134), Reduce(134), Reduce(134), Reduce(134), Reduce(134), Reduce(134), Reduce(134), Reduce(134), 
    Reduce(134), Reduce(134), Reduce(134), Reduce(134), Reduce(134), Reduce(134), Reduce(134), Reduce(134), 
    Reduce(134), Reduce(134), Reduce(134), Reduce(134), Reduce(134), Reduce(133), Reduce(133), Reduce(133), 
    Reduce(133), Reduce(133), Reduce(133), Reduce(133), Reduce(50), Reduce(133), Reduce(133), Error, 
    Shift(207), Reduce(197), Error, Error, Reduce(111), Error, Reduce(111), Reduce(111), Error, Error, 
    Error, Error, Error, Error, Error, Error, Error, Reduce(133), Reduce(133), Reduce(133), Reduce(133), 
    Reduce(133), Reduce(133), Reduce(133), Reduce(133), Reduce(133), Reduce(133), Reduce(133), Reduce(133), 
    Reduce(133), Reduce(133), Reduce(133), Reduce(133), Reduce(133), Reduce(133), Reduce(133), Reduce(133), 
    Reduce(133), Reduce(133), Reduce(133), Reduce(133), Reduce(133), Reduce(133), Reduce(133), Reduce(133), 
    Reduce(133), Reduce(133), Reduce(133), Reduce(112), Reduce(112), Reduce(112), Reduce(112), Reduce(112), 
    Reduce(112), Reduce(112), Error, Reduce(112), Reduce(112), Error, Error, Error, Error, Error, Error, 
    Error, Error, Error, Error, Error, Error, Error, Error, Error, Error, Error, Error, Reduce(112), 
    Reduce(112), Reduce(112), Reduce(112), Reduce(112), Reduce(112), Reduce(112), Reduce(112), Reduce(112), 
    Reduce(112), Reduce(112), Reduce(112), Reduce(112), Reduce(112), Reduce(112), Reduce(112), Reduce(112), 
    Reduce(112), Reduce(112), Reduce(112), Reduce(112), Reduce(112), Reduce(112), Reduce(112), Reduce(112), 
    Reduce(112), Reduce(112), Reduce(112), Reduce(112), Reduce(112), Reduce(112), Error, Reduce(110), 
    Error, Error, Error, Error, Reduce(110), Error, Reduce(110), Error, Reduce(110), Reduce(110), Error, 
    Reduce(110), Error, Reduce(53), Error, Error, Error, Error, Error, Error, Error, Error, Error, Error, 
    Error, Reduce(110), Error, Error, Error, Error, Reduce(111), Reduce(111), Reduce(111), Reduce(111), 
    Reduce(111), Reduce(111), Reduce(111), Error, Reduce(111), Reduce(111), Error, Error, Error, Error, 
    Error, Error, Error, Error, Error, Error, Error, Error, Error, Error, Error, Error, Error, Error, 
    Reduce(111), Reduce(111), Reduce(111), Reduce(111), Reduce(111), Reduce(111), Reduce(111), Reduce(111), 
    Reduce(111), Reduce(111), Reduce(111), Reduce(111), Reduce(111), Reduce(111), Reduce(111), Reduce(111), 
    Reduce(111), Reduce(111), Reduce(111), Reduce(111), Reduce(111), Reduce(111), Reduce(111), Reduce(111), 
    Reduce(111), Reduce(111), Reduce(111), Reduce(111), Reduce(111), Reduce(111), Reduce(111), Reduce(110), 
    Reduce(109), Reduce(110), Reduce(110), Error, Error, Reduce(109), Error, Reduce(109), Error, Reduce(109), 
    Reduce(109), Shift(78), Reduce(109), Error, Error, Error, Shift(79), Error, Shift(80), Error, Shift(81), 
    Shift(82), Error, Shift(83), Error, Error, Reduce(109), Error, Error, Error, Error, Error, Error, 
    Error, Error, Error, Error, Error, Error, Error, Error, Error, Error, Error, Error, Error, Error, 
    Error, Error, Error, Error, Error, Error, Error, Error, Error, Error, Error, Error, Reduce(108), 
    Error, Error, Reduce(198), Error, Reduce(108), Reduce(198), Reduce(108), Error, Reduce(108), Reduce(108), 
    Error, Reduce(108), Error, Error, Error, Error, Error, Error, Error, Reduce(198), Reduce(198), Error, 
    Error, Error, Reduce(198), Reduce(108), Error, Error, Error, Error, Reduce(109), Error, Reduce(109), 
    Reduce(109), Error, Error, Error, Error, Error, Error, Error, Shift(84), Error, Shift(305), Shift(85), 
    Error, Error, Error, Error, Error, Error, Error, Error, Error, Reduce(198), Error, Error, Error, 
    Reduce(107), Error, Error, Error, Error, Reduce(107), Error, Reduce(107), Error, Reduce(107), Reduce(107), 
    Error, Reduce(107), Error, Error, Error, Reduce(110), Reduce(110), Reduce(110), Reduce(110), Reduce(110), 
    Reduce(110), Reduce(110), Error, Reduce(110), Reduce(110), Reduce(107), Error, Reduce(198), Error, 
    Error, Reduce(108), Error, Reduce(108), Reduce(108), Error, Error, Error, Error, Error, Error, Error, 
    Error, Error, Reduce(110), Reduce(110), Reduce(110), Reduce(110), Reduce(110), Reduce(110), Reduce(110), 
    Reduce(110), Reduce(110), Reduce(110), Reduce(110), Reduce(110), Reduce(110), Reduce(110), Reduce(110), 
    Reduce(110), Reduce(110), Reduce(110), Reduce(110), Reduce(110), Reduce(110), Reduce(110), Reduce(110), 
    Reduce(110), Reduce(110), Reduce(110), Reduce(110), Reduce(110), Reduce(110), Reduce(110), Reduce(110), 
    Error, Error, Error, Error, Error, Error, Error, Error, Error, Error, Error, Error, Error, Error, 
    Error, Reduce(107), Reduce(122), Reduce(107), Reduce(107), Error, Error, Reduce(122), Error, Reduce(122), 
    Error, Reduce(122), Reduce(122), Error, Reduce(122), Error, Error, Error, Reduce(109), Reduce(109), 
    Reduce(109), Reduce(109), Reduce(109), Reduce(109), Reduce(109), Error, Reduce(109), Reduce(109), 
    Reduce(122), Shift(86), Error, Shift(87), Shift(88), Shift(89), Shift(90), Shift(91), Error, Shift(92), 
    Shift(93), Error, Error, Error, Error, Error, Error, Error, Reduce(109), Reduce(109), Reduce(109), 
    Reduce(109), Reduce(109), Reduce(109), Reduce(109), Reduce(109), Reduce(109), Reduce(109), Reduce(109), 
    Reduce(109), Reduce(109), Reduce(109), Reduce(109), Reduce(109), Reduce(109), Reduce(109), Reduce(109), 
    Reduce(109), Reduce(109), Reduce(109), Reduce(109), Reduce(109), Reduce(109), Reduce(109), Reduce(109), 
    Reduce(109), Reduce(109), Reduce(109), Reduce(109), Reduce(108), Reduce(108), Reduce(108), Reduce(108), 
    Reduce(108), Reduce(108), Reduce(108), Error, Reduce(108), Reduce(108), Error, Shift(207), Reduce(198), 
    Error, Error, Reduce(122), Error, Reduce(122), Reduce(122), Error, Error, Error, Error, Error, Error, 
    Error, Error, Error, Reduce(108), Reduce(108), Reduce(108), Reduce(108), Reduce(108), Reduce(108), 
    Reduce(108), Reduce(108), Reduce(108), Reduce(108), Reduce(108), Reduce(108), Reduce(108), Reduce(108), 
    Reduce(108), Reduce(108), Reduce(108), Reduce(108), Reduce(108), Reduce(108), Reduce(108), Reduce(108), 
    Reduce(108), Reduce(108), Reduce(108), Reduce(108), Reduce(108), Reduce(108), Reduce(108), Reduce(108), 
    Reduce(108), Reduce(107), Reduce(107), Reduce(107), Reduce(107), Reduce(107), Reduce(107), Reduce(107), 
    Error, Reduce(107), Reduce(107), Error, Error, Error, Error, Error, Error, Error, Error, Error, Error, 
    Error, Error, Error, Error, Error, Error, Error, Reduce(10), Reduce(107), Reduce(107), Reduce(107), 
    Reduce(107), Reduce(107), Reduce(107), Reduce(107), Reduce(107), Reduce(107), Reduce(107), Reduce(107), 
    Reduce(107), Reduce(107), Reduce(107), Reduce(107), Reduce(107), Reduce(107), Reduce(107), Reduce(107), 
    Reduce(107), Reduce(107), Reduce(107), Reduce(107), Reduce(107), Reduce(107), Reduce(107), Reduce(107), 
    Reduce(107), Reduce(107), Reduce(107), Reduce(107), Error, Reduce(10), Error, Error, Error, Error, 
    Reduce(10), Error, Reduce(10), Error, Reduce(10), Reduce(10), Error, Reduce(10), Error, Error, Error, 
    Error, Error, Error, Error, Error, Error, Error, Error, Error, Error, Reduce(10), Error, Error, Error, 
    Error, Reduce(122), Reduce(122), Reduce(122), Reduce(122), Reduce(122), Reduce(122), Reduce(122), 
    Error, Reduce(122), Reduce(122), Error, Error, Error, Error, Error, Error, Error, Error, Error, Error, 
    Error, Error, Error, Error, Error, Error, Error, Error, Reduce(122), Reduce(122), Reduce(122), Reduce(122), 
    Reduce(122), Reduce(122), Reduce(122), Reduce(122), Reduce(122), Reduce(122), Reduce(122), Reduce(122), 
    Reduce(122), Reduce(122), Reduce(122), Reduce(122), Reduce(122), Reduce(122), Reduce(122), Reduce(122), 
    Reduce(122), Reduce(122), Reduce(122), Reduce(122), Reduce(122), Reduce(122), Reduce(122), Reduce(122), 
    Reduce(122), Reduce(122), Reduce(122), Reduce(10), Reduce(119), Reduce(10), Reduce(10), Error, Error, 
    Reduce(119), Error, Reduce(119), Error, Reduce(119), Reduce(119), Shift(78), Reduce(119), Error, 
    Error, Error, Shift(79), Error, Shift(80), Reduce(130), Shift(81), Shift(82), Error, Shift(83), Error, 
    Error, Reduce(119), Error, Error, Error, Error, Error, Error, Error, Error, Error, Error, Error, 
    Error, Error, Error, Error, Error, Error, Error, Error, Error, Error, Error, Error, Error, Error, 
    Error, Error, Error, Error, Error, Error, Error, Reduce(118), Error, Error, Error, Error, Reduce(118), 
    Error, Reduce(118), Error, Reduce(118), Reduce(118), Error, Reduce(118), Error, Error, Error, Error, 
    Error, Error, Error, Error, Error, Error, Error, Error, Error, Reduce(118), Error, Error, Error, 
    Error, Reduce(119), Error, Reduce(119), Reduce(119), Error, Error, Error, Error, Error, Error, Error, 
    Error, Error, Error, Shift(85), Error, Error, Error, Error, Error, Error, Error, Error, Error, Error, 
    Error, Error, Error, Shift(78), Error, Error, Error, Error, Shift(79), Error, Shift(80), Error, Shift(81), 
    Shift(82), Error, Shift(83), Error, Error, Error, Reduce(10), Reduce(10), Reduce(10), Reduce(10), 
    Reduce(10), Reduce(10), Reduce(10), Error, Reduce(10), Reduce(10), Shift(120), Error, Error, Error, 
    Error, Reduce(118), Error, Reduce(118), Reduce(118), Error, Error, Error, Error, Error, Error, Error, 
    Error, Error, Reduce(10), Reduce(10), Reduce(10), Reduce(10), Reduce(10), Reduce(10), Reduce(10), 
    Reduce(10), Reduce(10), Reduce(10), Reduce(10), Reduce(10), Reduce(10), Reduce(10), Reduce(10), Reduce(10), 
    Reduce(10), Reduce(10), Reduce(10), Reduce(10), Reduce(10), Reduce(10), Error, Reduce(10), Reduce(10), 
    Reduce(10), Reduce(10), Reduce(10), Reduce(10), Reduce(10), Reduce(10), Error, Error, Error, Error, 
    Error, Error, Error, Error, Error, Error, Error, Error, Error, Error, Error, Shift(67), Reduce(121), 
    Shift(232), Shift(85), Error, Error, Reduce(121), Error, Reduce(121), Error, Reduce(121), Reduce(121), 
    Error, Reduce(121), Error, Error, Error, Reduce(119), Reduce(119), Reduce(119), Reduce(119), Reduce(119), 
    Reduce(119), Reduce(119), Error, Reduce(119), Reduce(119), Reduce(121), Shift(86), Error, Shift(87), 
    Shift(88), Shift(89), Shift(90), Shift(91), Error, Shift(92), Shift(93), Error, Error, Error, Error, 
    Error, Error, Error, Reduce(119), Reduce(119), Reduce(119), Reduce(119), Reduce(119), Reduce(119), 
    Reduce(119), Reduce(119), Reduce(119), Reduce(119), Reduce(119), Reduce(119), Reduce(119), Reduce(119), 
    Reduce(119), Reduce(119), Reduce(119), Reduce(119), Reduce(119), Reduce(119), Reduce(119), Reduce(119), 
    Error, Reduce(119), Reduce(119), Reduce(119), Reduce(119), Reduce(119), Reduce(119), Reduce(119), 
    Reduce(119), Reduce(118), Reduce(118), Reduce(118), Reduce(118), Reduce(118), Reduce(118), Reduce(118), 
    Error, Reduce(118), Reduce(118), Error, Error, Error, Error, Error, Reduce(121), Error, Reduce(121), 
    Reduce(121), Error, Error, Error, Error, Error, Error, Error, Error, Error, Reduce(118), Reduce(118), 
    Reduce(118), Reduce(118), Reduce(118), Reduce(118), Reduce(118), Reduce(118), Reduce(118), Reduce(118), 
    Reduce(118), Reduce(118), Reduce(118), Reduce(118), Reduce(118), Reduce(118), Reduce(118), Reduce(118), 
    Reduce(118), Reduce(118), Reduce(118), Reduce(118), Error, Reduce(118), Reduce(118), Reduce(118), 
    Reduce(118), Reduce(118), Reduce(118), Reduce(118), Reduce(118), Shift(122), Shift(2), Shift(87), 
    Shift(88), Shift(89), Shift(90), Shift(91), Error, Shift(92), Shift(93), Error, Error, Error, Error, 
    Error, Error, Error, Error, Error, Error, Error, Error, Error, Error, Error, Error, Error, Error, 
    Shift(3), Shift(4), Shift(5), Shift(6), Shift(7), Shift(8), Shift(9), Shift(10), Shift(11), Shift(12), 
    Shift(13), Shift(14), Shift(15), Shift(16), Shift(17), Shift(18), Shift(19), Shift(20), Shift(21), 
    Shift(123), Shift(124), Shift(125), Error, Shift(126), Shift(127), Shift(128), Shift(129), Shift(130), 
    Shift(131), Shift(132), Shift(133), Error, Reduce(120), Error, Error, Error, Error, Reduce(120), 
    Error, Reduce(120), Error, Reduce(120), Reduce(120), Error, Reduce(120), Error, Error, Shift(78), 
    Error, Error, Error, Error, Shift(79), Error, Shift(80), Error, Shift(81), Shift(82), Reduce(120), 
    Shift(83), Error, Error, Error, Reduce(121), Reduce(121), Reduce(121), Reduce(121), Reduce(121), 
    Reduce(121), Reduce(121), Error, Reduce(121), Reduce(121), Shift(120), Error, Error, Error, Error, 
    Error, Error, Error, Error, Error, Error, Error, Error, Error, Error, Error, Error, Error, Reduce(121), 
    Reduce(121), Reduce(121), Reduce(121), Reduce(121), Reduce(121), Reduce(121), Reduce(121), Reduce(121), 
    Reduce(121), Reduce(121), Reduce(121), Reduce(121), Reduce(121), Reduce(121), Reduce(121), Reduce(121), 
    Reduce(121), Reduce(121), Reduce(121), Reduce(121), Reduce(121), Error, Reduce(121), Reduce(121), 
    Reduce(121), Reduce(121), Reduce(121), Reduce(121), Reduce(121), Reduce(121), Reduce(120), Shift(78), 
    Reduce(120), Reduce(120), Error, Error, Shift(79), Error, Shift(80), Error, Shift(81), Shift(82), 
    Error, Shift(83), Error, Shift(67), Reduce(178), Error, Shift(85), Reduce(178), Error, Shift(194), 
    Reduce(178), Shift(195), Error, Error, Error, Shift(120), Error, Error, Error, Error, Error, Error, 
    Error, Error, Reduce(178), Reduce(178), Reduce(178), Error, Reduce(178), Reduce(178), Error, Error, 
    Error, Error, Error, Error, Error, Error, Error, Error, Error, Error, Error, Error, Error, Error, 
    Error, Reduce(145), Reduce(145), Error, Reduce(145), Reduce(145), Reduce(145), Reduce(145), Reduce(145), 
    Reduce(145), Reduce(145), Reduce(145), Error, Reduce(178), Reduce(178), Error, Error, Error, Error, 
    Error, Error, Error, Reduce(145), Reduce(145), Reduce(145), Reduce(145), Reduce(145), Reduce(145), 
    Error, Error, Error, Error, Error, Shift(67), Error, Shift(121), Shift(85), Error, Error, Error, 
    Error, Error, Error, Error, Reduce(178), Reduce(178), Error, Error, Error, Error, Error, Error, Error, 
    Error, Error, Reduce(145), Error, Reduce(145), Reduce(145), Error, Error, Error, Error, Shift(78), 
    Error, Error, Error, Error, Shift(79), Error, Shift(80), Reduce(153), Shift(81), Shift(82), Error, 
    Shift(83), Error, Reduce(120), Reduce(120), Reduce(120), Reduce(120), Reduce(120), Reduce(120), Reduce(120), 
    Error, Reduce(120), Reduce(120), Error, Reduce(145), Reduce(145), Error, Error, Shift(122), Error, 
    Shift(87), Shift(88), Shift(89), Shift(90), Shift(91), Error, Shift(92), Shift(93), Error, Error, 
    Error, Reduce(120), Reduce(120), Reduce(120), Reduce(120), Reduce(120), Reduce(120), Reduce(120), 
    Reduce(120), Reduce(120), Reduce(120), Reduce(120), Reduce(120), Reduce(120), Reduce(120), Reduce(120), 
    Reduce(120), Reduce(120), Reduce(120), Reduce(120), Reduce(120), Reduce(120), Reduce(120), Error, 
    Reduce(120), Reduce(120), Reduce(120), Reduce(120), Reduce(120), Reduce(120), Reduce(120), Reduce(120), 
    Error, Error, Error, Shift(123), Shift(124), Shift(125), Error, Shift(126), Shift(127), Shift(128), 
    Shift(129), Shift(130), Shift(131), Shift(132), Shift(133), Error, Error, Error, Error, Error, Shift(85), 
    Error, Error, Error, Error, Error, Error, Error, Error, Error, Error, Error, Shift(122), Shift(2), 
    Shift(87), Shift(88), Shift(89), Shift(90), Shift(91), Error, Shift(92), Shift(93), Error, Error, 
    Error, Error, Error, Error, Error, Error, Error, Error, Reduce(178), Reduce(178), Reduce(178), Reduce(178), 
    Reduce(178), Reduce(178), Reduce(178), Reduce(178), Shift(3), Shift(4), Shift(5), Shift(6), Shift(7), 
    Shift(8), Shift(9), Shift(10), Shift(11), Shift(12), Shift(13), Shift(14), Shift(15), Shift(16), 
    Shift(17), Shift(18), Shift(19), Shift(20), Shift(21), Shift(123), Shift(124), Shift(125), Error, 
    Shift(126), Shift(127), Shift(128), Shift(129), Shift(130), Shift(131), Shift(132), Shift(133), Reduce(145), 
    Error, Reduce(145), Reduce(145), Reduce(145), Reduce(145), Reduce(145), Reduce(145), Reduce(145), 
    Reduce(145), Reduce(145), Reduce(145), Reduce(145), Reduce(145), Reduce(145), Reduce(145), Reduce(145), 
    Reduce(145), Reduce(145), Reduce(145), Reduce(145), Reduce(145), Reduce(145), Reduce(139), Reduce(139), 
    Error, Reduce(139), Reduce(139), Reduce(139), Reduce(139), Reduce(139), Reduce(139), Reduce(139), 
    Reduce(139), Error, Error, Error, Error, Error, Error, Error, Error, Error, Error, Reduce(139), Reduce(139), 
    Reduce(139), Reduce(139), Reduce(139), Reduce(139), Reduce(144), Reduce(144), Error, Reduce(144), 
    Reduce(144), Reduce(144), Reduce(144), Reduce(144), Reduce(144), Reduce(144), Reduce(144), Shift(86), 
    Error, Shift(87), Shift(88), Shift(89), Shift(90), Shift(91), Error, Shift(92), Shift(93), Reduce(144), 
    Reduce(144), Reduce(144), Reduce(144), Reduce(144), Reduce(144), Reduce(139), Reduce(213), Reduce(139), 
    Reduce(139), Error, Error, Reduce(213), Error, Reduce(213), Error, Reduce(213), Reduce(213), Error, 
    Reduce(213), Reduce(147), Reduce(147), Error, Reduce(147), Reduce(147), Reduce(147), Reduce(147), 
    Reduce(147), Reduce(147), Reduce(147), Reduce(147), Error, Error, Reduce(144), Error, Reduce(144), 
    Reduce(144), Error, Error, Reduce(139), Reduce(139), Reduce(147), Reduce(147), Reduce(147), Reduce(147), 
    Reduce(147), Reduce(147), Reduce(148), Reduce(148), Error, Reduce(148), Reduce(148), Reduce(148), 
    Reduce(148), Reduce(148), Reduce(148), Reduce(148), Reduce(148), Error, Error, Error, Error, Error, 
    Error, Error, Error, Reduce(144), Reduce(144), Reduce(148), Reduce(148), Reduce(148), Reduce(148), 
    Reduce(148), Reduce(148), Reduce(147), Shift(78), Reduce(147), Reduce(147), Error, Error, Shift(79), 
    Error, Shift(80), Error, Shift(81), Shift(82), Error, Shift(83), Reduce(150), Reduce(150), Error, 
    Reduce(150), Reduce(150), Reduce(150), Reduce(150), Reduce(150), Reduce(150), Reduce(150), Reduce(150), 
    Error, Reduce(213), Reduce(148), Error, Reduce(148), Reduce(148), Error, Error, Reduce(147), Reduce(147), 
    Reduce(150), Reduce(150), Reduce(150), Reduce(150), Reduce(150), Reduce(150), Reduce(149), Reduce(149), 
    Error, Reduce(149), Reduce(149), Reduce(149), Reduce(149), Reduce(149), Reduce(149), Reduce(149), 
    Reduce(149), Error, Error, Error, Error, Error, Error, Error, Error, Reduce(148), Reduce(148), Reduce(149), 
    Reduce(149), Reduce(149), Reduce(149), Reduce(149), Reduce(149), Reduce(150), Error, Reduce(150), 
    Reduce(150), Error, Error, Error, Error, Reduce(212), Error, Error, Error, Error, Reduce(212), Error, 
    Reduce(212), Error, Reduce(212), Reduce(212), Error, Reduce(212), Error, Error, Shift(84), Error, 
    Error, Shift(85), Reduce(149), Error, Reduce(149), Reduce(149), Error, Error, Reduce(150), Reduce(150), 
    Error, Shift(171), Error, Reduce(139), Reduce(139), Reduce(139), Reduce(139), Reduce(139), Reduce(139), 
    Reduce(139), Reduce(139), Reduce(139), Reduce(139), Reduce(139), Reduce(139), Reduce(139), Reduce(139), 
    Reduce(139), Reduce(139), Reduce(139), Reduce(139), Reduce(139), Reduce(139), Reduce(139), Error, 
    Reduce(149), Reduce(149), Error, Reduce(144), Error, Reduce(144), Reduce(144), Reduce(144), Reduce(144), 
    Reduce(144), Reduce(144), Reduce(144), Reduce(144), Reduce(144), Reduce(144), Reduce(144), Reduce(144), 
    Reduce(144), Reduce(144), Reduce(144), Reduce(144), Reduce(144), Reduce(144), Reduce(144), Reduce(144), 
    Reduce(144), Error, Error, Error, Error, Reduce(213), Error, Reduce(213), Reduce(213), Reduce(213), 
    Reduce(213), Reduce(213), Error, Reduce(213), Reduce(213), Error, Reduce(212), Error, Error, Error, 
    Error, Reduce(147), Reduce(147), Reduce(147), Reduce(147), Reduce(147), Reduce(147), Reduce(147), 
    Reduce(147), Reduce(147), Reduce(147), Reduce(147), Reduce(147), Reduce(147), Reduce(147), Reduce(147), 
    Reduce(147), Reduce(147), Reduce(147), Reduce(147), Reduce(147), Reduce(147), Error, Error, Error, 
    Error, Error, Error, Reduce(148), Reduce(148), Reduce(148), Reduce(148), Reduce(148), Reduce(148), 
    Reduce(148), Reduce(148), Reduce(148), Reduce(148), Reduce(148), Reduce(148), Reduce(148), Reduce(148), 
    Reduce(148), Reduce(148), Reduce(148), Reduce(148), Reduce(148), Reduce(148), Reduce(148), Error, 
    Error, Error, Error, Shift(86), Error, Shift(87), Shift(88), Shift(89), Shift(90), Shift(91), Error, 
    Shift(92), Shift(93), Error, Error, Error, Error, Error, Error, Reduce(150), Reduce(150), Reduce(150), 
    Reduce(150), Reduce(150), Reduce(150), Reduce(150), Reduce(150), Reduce(150), Reduce(150), Reduce(150), 
    Reduce(150), Reduce(150), Reduce(150), Reduce(150), Reduce(150), Reduce(150), Reduce(150), Reduce(150), 
    Reduce(150), Reduce(150), Error, Error, Error, Error, Error, Error, Reduce(149), Reduce(149), Reduce(149), 
    Reduce(149), Reduce(149), Reduce(149), Reduce(149), Reduce(149), Reduce(149), Reduce(149), Reduce(149), 
    Reduce(149), Reduce(149), Reduce(149), Reduce(149), Reduce(149), Reduce(149), Reduce(149), Reduce(149), 
    Reduce(149), Reduce(149), Reduce(140), Reduce(140), Error, Reduce(140), Reduce(140), Reduce(140), 
    Reduce(140), Reduce(140), Reduce(140), Reduce(140), Reduce(140), Reduce(212), Error, Reduce(212), 
    Reduce(212), Reduce(212), Reduce(212), Reduce(212), Error, Reduce(212), Reduce(212), Reduce(140), 
    Reduce(140), Reduce(140), Reduce(140), Reduce(140), Reduce(140), Reduce(152), Reduce(152), Error, 
    Reduce(152), Reduce(152), Reduce(152), Reduce(152), Reduce(152), Reduce(152), Reduce(152), Reduce(152), 
    Error, Error, Error, Error, Error, Error, Error, Error, Error, Error, Reduce(152), Reduce(152), Reduce(152), 
    Reduce(152), Reduce(152), Reduce(152), Reduce(140), Reduce(211), Reduce(140), Reduce(140), Error, 
    Error, Reduce(211), Error, Reduce(211), Error, Reduce(211), Reduce(211), Error, Reduce(211), Reduce(151), 
    Reduce(151), Error, Reduce(151), Reduce(151), Reduce(151), Reduce(151), Reduce(151), Reduce(151), 
    Reduce(151), Reduce(151), Error, Error, Reduce(152), Error, Reduce(152), Reduce(152), Error, Error, 
    Reduce(140), Reduce(140), Reduce(151), Reduce(151), Reduce(151), Reduce(151), Reduce(151), Reduce(151), 
    Reduce(157), Reduce(157), Error, Shift(172), Reduce(157), Reduce(157), Reduce(157), Reduce(157), 
    Reduce(157), Shift(173), Reduce(157), Error, Error, Error, Error, Error, Error, Error, Error, Reduce(152), 
    Reduce(152), Reduce(157), Reduce(157), Reduce(157), Reduce(157), Reduce(157), Reduce(157), Reduce(151), 
    Reduce(210), Reduce(151), Reduce(151), Error, Error, Reduce(210), Error, Reduce(210), Error, Reduce(210), 
    Reduce(210), Error, Reduce(210), Reduce(138), Reduce(138), Error, Reduce(138), Reduce(138), Reduce(138), 
    Reduce(138), Reduce(138), Reduce(138), Reduce(138), Reduce(138), Error, Reduce(211), Shift(174), 
    Error, Reduce(157), Reduce(157), Error, Error, Reduce(151), Reduce(151), Reduce(138), Reduce(138), 
    Reduce(138), Reduce(138), Reduce(138), Reduce(138), Reduce(146), Reduce(146), Error, Reduce(146), 
    Reduce(146), Reduce(146), Reduce(146), Reduce(146), Reduce(146), Reduce(146), Reduce(146), Error, 
    Error, Error, Error, Error, Error, Error, Error, Reduce(157), Reduce(157), Reduce(146), Reduce(146), 
    Reduce(146), Reduce(146), Reduce(146), Reduce(146), Reduce(138), Error, Reduce(138), Reduce(138), 
    Error, Error, Error, Error, Reduce(209), Error, Error, Error, Error, Reduce(209), Error, Reduce(209), 
    Error, Reduce(209), Reduce(209), Error, Reduce(209), Error, Error, Error, Error, Error, Reduce(210), 
    Reduce(146), Error, Reduce(146), Reduce(146), Error, Error, Reduce(138), Reduce(138), Error, Error, 
    Error, Reduce(140), Reduce(140), Reduce(140), Reduce(140), Reduce(140), Reduce(140), Reduce(140), 
    Reduce(140), Reduce(140), Reduce(140), Reduce(140), Reduce(140), Reduce(140), Reduce(140), Reduce(140), 
    Reduce(140), Reduce(140), Reduce(140), Reduce(140), Reduce(140), Reduce(140), Error, Reduce(146), 
    Reduce(146), Error, Error, Error, Reduce(152), Reduce(152), Reduce(152), Reduce(152), Reduce(152), 
    Reduce(152), Reduce(152), Reduce(152), Reduce(152), Reduce(152), Reduce(152), Reduce(152), Reduce(152), 
    Reduce(152), Reduce(152), Reduce(152), Reduce(152), Reduce(152), Reduce(152), Reduce(152), Reduce(152), 
    Error, Error, Error, Error, Reduce(211), Error, Reduce(211), Reduce(211), Reduce(211), Reduce(211), 
    Reduce(211), Error, Reduce(211), Reduce(211), Error, Reduce(209), Error, Error, Error, Error, Reduce(151), 
    Reduce(151), Reduce(151), Reduce(151), Reduce(151), Reduce(151), Reduce(151), Reduce(151), Reduce(151), 
    Reduce(151), Reduce(151), Reduce(151), Reduce(151), Reduce(151), Reduce(151), Reduce(151), Reduce(151), 
    Reduce(151), Reduce(151), Reduce(151), Reduce(151), Error, Error, Error, Error, Error, Error, Shift(175), 
    Shift(176), Shift(177), Reduce(157), Reduce(157), Reduce(157), Reduce(157), Reduce(157), Reduce(157), 
    Reduce(157), Reduce(157), Reduce(157), Reduce(157), Reduce(157), Reduce(157), Reduce(157), Reduce(157), 
    Reduce(157), Reduce(157), Reduce(157), Reduce(157), Error, Error, Error, Error, Reduce(210), Error, 
    Reduce(210), Reduce(210), Reduce(210), Reduce(210), Reduce(210), Error, Reduce(210), Reduce(210), 
    Error, Error, Error, Error, Error, Error, Reduce(138), Reduce(138), Reduce(138), Reduce(138), Reduce(138), 
    Reduce(138), Reduce(138), Reduce(138), Reduce(138), Reduce(138), Reduce(138), Reduce(138), Reduce(138), 
    Reduce(138), Reduce(138), Reduce(138), Reduce(138), Reduce(138), Reduce(138), Reduce(138), Reduce(138), 
    Error, Error, Error, Error, Error, Error, Reduce(146), Reduce(146), Reduce(146), Reduce(146), Reduce(146), 
    Reduce(146), Reduce(146), Reduce(146), Reduce(146), Reduce(146), Reduce(146), Reduce(146), Reduce(146), 
    Reduce(146), Reduce(146), Reduce(146), Reduce(146), Reduce(146), Reduce(146), Reduce(146), Reduce(146), 
    Reduce(143), Reduce(143), Error, Reduce(143), Reduce(143), Reduce(143), Reduce(143), Reduce(143), 
    Reduce(143), Reduce(143), Reduce(143), Reduce(209), Error, Reduce(209), Reduce(209), Reduce(209), 
    Reduce(209), Reduce(209), Error, Reduce(209), Reduce(209), Reduce(143), Reduce(143), Reduce(143), 
    Reduce(143), Reduce(143), Reduce(143), Reduce(142), Reduce(142), Error, Reduce(142), Reduce(142), 
    Reduce(142), Reduce(142), Reduce(142), Reduce(142), Reduce(142), Reduce(142), Error, Error, Error, 
    Error, Error, Error, Error, Error, Error, Error, Reduce(142), Reduce(142), Reduce(142), Reduce(142), 
    Reduce(142), Reduce(142), Reduce(143), Error, Reduce(143), Reduce(143), Error, Error, Error, Error, 
    Error, Error, Error, Error, Error, Error, Reduce(141), Reduce(141), Error, Reduce(141), Reduce(141), 
    Reduce(141), Reduce(141), Reduce(141), Reduce(141), Reduce(141), Reduce(141), Error, Error, Reduce(142), 
    Error, Reduce(142), Reduce(142), Error, Error, Reduce(143), Reduce(143), Reduce(141), Reduce(141), 
    Reduce(141), Reduce(141), Reduce(141), Reduce(141), Reduce(137), Reduce(137), Error, Reduce(137), 
    Reduce(137), Reduce(137), Reduce(137), Reduce(137), Reduce(137), Reduce(137), Reduce(137), Error, 
    Error, Error, Error, Error, Error, Error, Error, Reduce(142), Reduce(142), Reduce(137), Reduce(137), 
    Reduce(137), Reduce(137), Reduce(137), Reduce(137), Reduce(141), Reduce(208), Reduce(141), Reduce(141), 
    Error, Error, Reduce(208), Error, Reduce(208), Error, Reduce(208), Reduce(208), Error, Reduce(208), 
    Reduce(137), Reduce(137), Error, Reduce(137), Error, Reduce(137), Reduce(137), Reduce(137), Reduce(137), 
    Reduce(137), Reduce(137), Error, Error, Reduce(137), Error, Reduce(137), Reduce(137), Error, Error, 
    Reduce(141), Reduce(141), Shift(219), Reduce(137), Reduce(137), Reduce(137), Reduce(137), Reduce(137), 
    Reduce(162), Reduce(162), Error, Error, Reduce(162), Reduce(162), Reduce(162), Reduce(162), Reduce(162), 
    Error, Reduce(162), Error, Error, Error, Error, Error, Error, Error, Error, Reduce(137), Reduce(137), 
    Reduce(162), Reduce(162), Reduce(162), Reduce(162), Reduce(162), Reduce(162), Reduce(137), Error, 
    Error, Reduce(137), Error, Error, Shift(78), Error, Error, Error, Error, Shift(79), Error, Shift(80), 
    Error, Shift(81), Shift(82), Error, Shift(83), Error, Error, Error, Error, Error, Error, Error, Reduce(208), 
    Error, Error, Reduce(162), Reduce(162), Error, Reduce(130), Reduce(137), Error, Error, Error, Error, 
    Reduce(143), Reduce(143), Reduce(143), Reduce(143), Reduce(143), Reduce(143), Reduce(143), Reduce(143), 
    Reduce(143), Reduce(143), Reduce(143), Reduce(143), Reduce(143), Reduce(143), Reduce(143), Reduce(143), 
    Reduce(143), Reduce(143), Reduce(143), Reduce(143), Reduce(143), Error, Reduce(162), Reduce(162), 
    Error, Error, Error, Reduce(142), Reduce(142), Reduce(142), Reduce(142), Reduce(142), Reduce(142), 
    Reduce(142), Reduce(142), Reduce(142), Reduce(142), Reduce(142), Reduce(142), Reduce(142), Reduce(142), 
    Reduce(142), Reduce(142), Reduce(142), Reduce(142), Reduce(142), Reduce(142), Reduce(142), Error, 
    Error, Error, Error, Error, Error, Error, Error, Error, Error, Error, Error, Error, Shift(85), Error, 
    Error, Error, Error, Error, Error, Reduce(141), Reduce(141), Reduce(141), Reduce(141), Reduce(141), 
    Reduce(141), Reduce(141), Reduce(141), Reduce(141), Reduce(141), Reduce(141), Reduce(141), Reduce(141), 
    Reduce(141), Reduce(141), Reduce(141), Reduce(141), Reduce(141), Reduce(141), Reduce(141), Reduce(141), 
    Error, Error, Error, Error, Error, Error, Reduce(137), Reduce(137), Reduce(137), Reduce(137), Reduce(137), 
    Reduce(137), Reduce(137), Reduce(137), Reduce(137), Reduce(137), Reduce(137), Reduce(137), Reduce(137), 
    Reduce(137), Reduce(137), Reduce(137), Reduce(137), Reduce(137), Reduce(137), Reduce(137), Reduce(137), 
    Error, Error, Error, Error, Reduce(208), Error, Reduce(208), Reduce(208), Reduce(208), Reduce(208), 
    Reduce(208), Error, Reduce(208), Reduce(208), Error, Error, Error, Error, Error, Error, Reduce(137), 
    Reduce(137), Reduce(137), Reduce(137), Reduce(137), Reduce(137), Reduce(137), Reduce(137), Reduce(137), 
    Reduce(137), Reduce(137), Reduce(137), Reduce(137), Reduce(137), Reduce(137), Reduce(137), Reduce(137), 
    Reduce(137), Reduce(137), Reduce(137), Reduce(137), Error, Error, Error, Error, Error, Error, Error, 
    Error, Error, Reduce(162), Reduce(162), Reduce(162), Reduce(162), Reduce(162), Reduce(162), Reduce(162), 
    Reduce(162), Reduce(162), Reduce(162), Reduce(162), Reduce(162), Reduce(162), Reduce(162), Reduce(162), 
    Reduce(162), Reduce(162), Reduce(162), Reduce(169), Reduce(169), Error, Error, Reduce(169), Reduce(169), 
    Reduce(169), Reduce(169), Reduce(169), Shift(86), Reduce(169), Shift(87), Shift(88), Shift(89), Shift(90), 
    Shift(91), Error, Shift(92), Shift(93), Error, Error, Reduce(169), Reduce(169), Reduce(169), Reduce(169), 
    Reduce(169), Reduce(169), Reduce(160), Reduce(160), Error, Error, Reduce(160), Reduce(160), Reduce(160), 
    Reduce(160), Reduce(160), Error, Reduce(160), Error, Error, Error, Error, Error, Error, Error, Error, 
    Error, Error, Reduce(160), Reduce(160), Reduce(160), Reduce(160), Reduce(160), Reduce(160), Reduce(159), 
    Reduce(159), Reduce(169), Reduce(169), Reduce(159), Reduce(159), Reduce(159), Reduce(159), Reduce(159), 
    Error, Reduce(159), Error, Error, Error, Error, Error, Error, Error, Error, Error, Error, Reduce(159), 
    Reduce(159), Reduce(159), Reduce(159), Reduce(159), Reduce(159), Error, Error, Reduce(160), Reduce(160), 
    Error, Error, Reduce(169), Reduce(169), Error, Error, Error, Error, Error, Error, Reduce(158), Reduce(158), 
    Error, Error, Reduce(158), Reduce(158), Reduce(158), Reduce(158), Reduce(158), Error, Reduce(158), 
    Error, Error, Error, Error, Reduce(159), Reduce(159), Error, Error, Reduce(160), Reduce(160), Reduce(158), 
    Reduce(158), Reduce(158), Reduce(158), Reduce(158), Reduce(158), Reduce(161), Reduce(161), Error, 
    Error, Reduce(161), Reduce(161), Reduce(161), Reduce(161), Reduce(161), Error, Reduce(161), Error, 
    Error, Error, Error, Error, Error, Error, Error, Reduce(159), Reduce(159), Reduce(161), Reduce(161), 
    Reduce(161), Reduce(161), Reduce(161), Reduce(161), Error, Error, Reduce(158), Reduce(158), Error, 
    Error, Error, Error, Error, Error, Error, Error, Error, Error, Reduce(170), Reduce(170), Error, Error, 
    Reduce(170), Reduce(170), Reduce(170), Reduce(170), Reduce(170), Error, Reduce(170), Error, Error, 
    Error, Error, Reduce(161), Reduce(161), Error, Error, Reduce(158), Reduce(158), Reduce(170), Reduce(170), 
    Reduce(170), Reduce(170), Reduce(170), Reduce(170), Error, Error, Error, Error, Error, Error, Error, 
    Error, Error, Error, Error, Error, Error, Error, Error, Error, Error, Error, Error, Reduce(161), 
    Reduce(161), Error, Error, Error, Error, Error, Shift(78), Error, Error, Reduce(170), Reduce(170), 
    Shift(79), Error, Shift(80), Error, Shift(81), Shift(82), Error, Shift(83), Error, Error, Reduce(169), 
    Reduce(169), Reduce(169), Reduce(169), Reduce(169), Reduce(169), Reduce(169), Reduce(169), Reduce(169), 
    Reduce(169), Reduce(169), Reduce(169), Reduce(169), Reduce(169), Reduce(169), Reduce(169), Reduce(169), 
    Reduce(169), Error, Reduce(170), Reduce(170), Error, Error, Error, Error, Error, Error, Reduce(160), 
    Reduce(160), Reduce(160), Reduce(160), Reduce(160), Reduce(160), Reduce(160), Reduce(160), Reduce(160), 
    Reduce(160), Reduce(160), Reduce(160), Reduce(160), Reduce(160), Reduce(160), Reduce(160), Reduce(160), 
    Reduce(160), Error, Error, Error, Error, Error, Error, Error, Error, Error, Reduce(159), Reduce(159), 
    Reduce(159), Reduce(159), Reduce(159), Reduce(159), Reduce(159), Reduce(159), Reduce(159), Reduce(159), 
    Reduce(159), Reduce(159), Reduce(159), Reduce(159), Reduce(159), Reduce(159), Reduce(159), Reduce(159), 
    Error, Error, Error, Error, Error, Reduce(81), Shift(85), Error, Error, Error, Error, Error, Error, 
    Error, Error, Error, Error, Error, Error, Error, Error, Error, Error, Reduce(158), Reduce(158), Reduce(158), 
    Reduce(158), Reduce(158), Reduce(158), Reduce(158), Reduce(158), Reduce(158), Reduce(158), Reduce(158), 
    Reduce(158), Reduce(158), Reduce(158), Reduce(158), Reduce(158), Reduce(158), Reduce(158), Error, 
    Error, Error, Error, Error, Error, Error, Error, Error, Reduce(161), Reduce(161), Reduce(161), Reduce(161), 
    Reduce(161), Reduce(161), Reduce(161), Reduce(161), Reduce(161), Reduce(161), Reduce(161), Reduce(161), 
    Reduce(161), Reduce(161), Reduce(161), Reduce(161), Reduce(161), Reduce(161), Error, Reduce(42), 
    Reduce(42), Reduce(42), Error, Reduce(42), Error, Error, Error, Error, Error, Error, Error, Error, 
    Error, Error, Error, Error, Error, Reduce(42), Reduce(42), Error, Error, Reduce(170), Reduce(170), 
    Reduce(170), Reduce(170), Reduce(170), Reduce(170), Reduce(170), Reduce(170), Reduce(170), Reduce(170), 
    Reduce(170), Reduce(170), Reduce(170), Reduce(170), Reduce(170), Reduce(170), Reduce(170), Reduce(170), 
    Reduce(170), Reduce(170), Error, Error, Reduce(170), Reduce(170), Reduce(170), Reduce(170), Reduce(170), 
    Error, Reduce(170), Reduce(42), Error, Error, Error, Error, Error, Error, Error, Error, Error, Reduce(170), 
    Reduce(170), Reduce(170), Shift(178), Reduce(170), Reduce(170), Error, Error, Shift(86), Shift(2), 
    Shift(87), Shift(88), Shift(89), Shift(90), Shift(91), Error, Shift(92), Shift(93), Error, Reduce(60), 
    Reduce(60), Reduce(60), Reduce(46), Reduce(60), Error, Error, Error, Error, Error, Error, Error, 
    Error, Error, Error, Error, Reduce(170), Reduce(170), Reduce(60), Reduce(60), Error, Error, Shift(8), 
    Shift(9), Shift(10), Shift(11), Shift(12), Shift(13), Shift(14), Shift(15), Shift(16), Shift(17), 
    Shift(18), Shift(19), Shift(20), Shift(21), Error, Error, Error, Error, Reduce(184), Error, Error, 
    Reduce(184), Error, Error, Reduce(184), Reduce(170), Reduce(170), Reduce(76), Reduce(76), Reduce(60), 
    Error, Reduce(76), Error, Error, Error, Error, Error, Error, Reduce(184), Reduce(184), Reduce(184), 
    Error, Reduce(184), Reduce(184), Error, Reduce(76), Reduce(76), Error, Reduce(76), Reduce(77), Reduce(77), 
    Shift(147), Shift(2), Reduce(77), Error, Error, Error, Error, Error, Error, Error, Reduce(46), Error, 
    Error, Error, Error, Error, Reduce(77), Reduce(77), Error, Reduce(77), Error, Error, Reduce(184), 
    Reduce(184), Error, Error, Error, Reduce(76), Shift(3), Shift(4), Shift(5), Shift(6), Shift(7), Shift(8), 
    Shift(9), Shift(10), Shift(11), Shift(12), Shift(13), Shift(14), Shift(15), Shift(16), Shift(17), 
    Shift(18), Shift(19), Shift(20), Shift(21), Error, Error, Reduce(77), Error, Error, Error, Reduce(184), 
    Reduce(184), Error, Error, Error, Error, Reduce(76), Error, Error, Error, Shift(78), Error, Reduce(42), 
    Reduce(42), Error, Shift(79), Error, Shift(80), Error, Shift(81), Shift(82), Error, Shift(83), Error, 
    Error, Error, Error, Error, Reduce(77), Error, Error, Error, Error, Error, Error, Error, Shift(230), 
    Error, Error, Error, Reduce(42), Reduce(42), Reduce(42), Reduce(42), Reduce(42), Reduce(42), Reduce(42), 
    Reduce(42), Reduce(42), Reduce(42), Reduce(42), Reduce(42), Reduce(42), Reduce(42), Reduce(42), Reduce(42), 
    Reduce(42), Reduce(42), Reduce(42), Error, Error, Error, Error, Error, Error, Reduce(170), Reduce(170), 
    Reduce(170), Reduce(170), Reduce(170), Reduce(170), Reduce(170), Reduce(170), Shift(179), Shift(180), 
    Shift(181), Shift(182), Shift(183), Shift(184), Shift(185), Shift(186), Shift(187), Shift(188), Error, 
    Error, Error, Error, Error, Error, Error, Error, Error, Reduce(60), Reduce(60), Error, Error, Error, 
    Error, Error, Error, Error, Error, Error, Shift(85), Error, Error, Error, Error, Error, Error, Error, 
    Error, Error, Error, Error, Error, Error, Error, Error, Error, Reduce(60), Reduce(60), Reduce(60), 
    Reduce(60), Reduce(60), Reduce(60), Reduce(60), Reduce(60), Reduce(60), Reduce(60), Reduce(60), Reduce(60), 
    Reduce(60), Reduce(60), Reduce(60), Reduce(60), Reduce(60), Reduce(60), Reduce(60), Reduce(78), Reduce(78), 
    Error, Reduce(76), Reduce(78), Shift(196), Shift(197), Reduce(184), Reduce(184), Reduce(184), Reduce(184), 
    Reduce(184), Reduce(184), Error, Error, Error, Error, Error, Reduce(78), Reduce(78), Error, Reduce(78), 
    Error, Error, Error, Reduce(77), Error, Error, Error, Error, Reduce(76), Reduce(76), Reduce(76), 
    Reduce(76), Reduce(76), Reduce(76), Reduce(76), Reduce(76), Reduce(76), Reduce(76), Reduce(76), Reduce(76), 
    Reduce(76), Reduce(76), Reduce(76), Reduce(76), Reduce(76), Reduce(76), Reduce(76), Error, Error, 
    Reduce(78), Reduce(77), Reduce(77), Reduce(77), Reduce(77), Reduce(77), Reduce(77), Reduce(77), Reduce(77), 
    Reduce(77), Reduce(77), Reduce(77), Reduce(77), Reduce(77), Reduce(77), Reduce(77), Reduce(77), Reduce(77), 
    Reduce(77), Reduce(77), Reduce(41), Reduce(41), Reduce(41), Error, Reduce(41), Error, Error, Error, 
    Error, Error, Error, Error, Reduce(78), Error, Error, Error, Error, Error, Reduce(41), Reduce(41), 
    Reduce(59), Reduce(59), Reduce(59), Error, Reduce(59), Shift(86), Error, Shift(87), Shift(88), Shift(89), 
    Shift(90), Shift(91), Error, Shift(92), Shift(93), Error, Error, Error, Reduce(59), Reduce(59), Error, 
    Error, Error, Error, Error, Error, Reduce(75), Reduce(75), Error, Error, Reduce(75), Reduce(41), 
    Error, Error, Reduce(189), Error, Error, Reduce(189), Error, Error, Reduce(189), Error, Error, Error, 
    Reduce(75), Reduce(75), Error, Reduce(75), Shift(72), Reduce(65), Error, Reduce(59), Reduce(65), 
    Error, Reduce(189), Reduce(189), Error, Error, Error, Reduce(189), Error, Error, Error, Error, Error, 
    Error, Reduce(65), Reduce(65), Error, Reduce(65), Error, Error, Error, Error, Reduce(37), Reduce(37), 
    Reduce(37), Reduce(75), Reduce(37), Error, Error, Error, Error, Error, Error, Error, Error, Error, 
    Error, Reduce(189), Reduce(189), Error, Reduce(37), Reduce(37), Reduce(74), Reduce(74), Error, Error, 
    Reduce(74), Shift(73), Error, Error, Error, Error, Error, Error, Error, Error, Error, Reduce(75), 
    Error, Error, Reduce(74), Reduce(74), Error, Reduce(74), Error, Error, Error, Error, Reduce(189), 
    Reduce(189), Error, Error, Error, Reduce(37), Error, Error, Reduce(78), Shift(204), Error, Reduce(65), 
    Reduce(191), Error, Error, Reduce(191), Error, Error, Error, Error, Error, Error, Error, Error, Error, 
    Reduce(74), Error, Error, Error, Reduce(191), Reduce(191), Error, Error, Error, Reduce(191), Reduce(78), 
    Reduce(78), Reduce(78), Reduce(78), Reduce(78), Reduce(78), Reduce(78), Reduce(78), Reduce(78), Reduce(78), 
    Reduce(78), Reduce(78), Reduce(78), Reduce(78), Reduce(78), Reduce(78), Reduce(78), Reduce(78), Reduce(78), 
    Reduce(36), Reduce(36), Reduce(36), Reduce(74), Reduce(36), Error, Error, Error, Error, Error, Reduce(191), 
    Reduce(191), Error, Error, Error, Error, Error, Error, Reduce(36), Reduce(36), Error, Error, Error, 
    Error, Reduce(41), Reduce(41), Error, Error, Error, Error, Error, Error, Error, Error, Error, Error, 
    Error, Error, Error, Error, Error, Reduce(191), Reduce(191), Error, Reduce(59), Reduce(59), Error, 
    Error, Error, Error, Error, Reduce(36), Reduce(41), Reduce(41), Reduce(41), Reduce(41), Reduce(41), 
    Reduce(41), Reduce(41), Reduce(41), Reduce(41), Reduce(41), Reduce(41), Reduce(41), Reduce(41), Reduce(41), 
    Reduce(41), Reduce(41), Reduce(41), Reduce(41), Reduce(41), Reduce(75), Reduce(59), Reduce(59), Reduce(59), 
    Reduce(59), Reduce(59), Reduce(59), Reduce(59), Reduce(59), Reduce(59), Reduce(59), Reduce(59), Reduce(59), 
    Reduce(59), Reduce(59), Reduce(59), Reduce(59), Reduce(59), Reduce(59), Reduce(59), Error, Error, 
    Reduce(65), Shift(202), Shift(203), Reduce(189), Reduce(189), Reduce(75), Reduce(75), Reduce(75), 
    Reduce(75), Reduce(75), Reduce(75), Reduce(75), Reduce(75), Reduce(75), Reduce(75), Reduce(75), Reduce(75), 
    Reduce(75), Reduce(75), Reduce(75), Reduce(75), Reduce(75), Reduce(75), Reduce(75), Error, Reduce(37), 
    Reduce(37), Reduce(65), Reduce(65), Reduce(65), Reduce(65), Reduce(65), Reduce(65), Reduce(65), Reduce(65), 
    Reduce(65), Reduce(65), Reduce(65), Reduce(65), Reduce(65), Reduce(65), Reduce(65), Reduce(65), Reduce(65), 
    Reduce(65), Reduce(65), Reduce(74), Error, Error, Error, Error, Error, Error, Reduce(37), Reduce(37), 
    Reduce(37), Reduce(37), Reduce(37), Reduce(37), Reduce(37), Reduce(37), Reduce(37), Reduce(37), Reduce(37), 
    Reduce(37), Reduce(37), Reduce(37), Reduce(37), Reduce(37), Reduce(37), Reduce(37), Reduce(37), Error, 
    Reduce(74), Reduce(74), Reduce(74), Reduce(74), Reduce(74), Reduce(74), Reduce(74), Reduce(74), Reduce(74), 
    Reduce(74), Reduce(74), Reduce(74), Reduce(74), Reduce(74), Reduce(74), Reduce(74), Reduce(74), Reduce(74), 
    Reduce(74), Reduce(40), Reduce(40), Reduce(40), Error, Reduce(40), Error, Reduce(191), Reduce(191), 
    Error, Error, Error, Error, Error, Error, Error, Error, Error, Error, Reduce(40), Reduce(40), Error, 
    Error, Error, Error, Error, Error, Error, Error, Error, Error, Error, Error, Error, Reduce(36), Reduce(36), 
    Error, Error, Error, Error, Error, Error, Error, Error, Error, Error, Error, Error, Error, Error, 
    Error, Error, Reduce(40), Error, Error, Error, Error, Error, Error, Error, Error, Error, Reduce(36), 
    Reduce(36), Reduce(36), Reduce(36), Reduce(36), Reduce(36), Reduce(36), Reduce(36), Reduce(36), Reduce(36), 
    Reduce(36), Reduce(36), Reduce(36), Reduce(36), Reduce(36), Reduce(36), Reduce(36), Reduce(36), Reduce(36), 
    Reduce(39), Reduce(39), Reduce(39), Error, Reduce(39), Error, Error, Error, Error, Error, Error, 
    Error, Error, Error, Error, Error, Error, Error, Reduce(39), Reduce(39), Reduce(27), Reduce(27), 
    Reduce(27), Error, Reduce(27), Error, Error, Error, Error, Error, Error, Error, Error, Error, Error, 
    Error, Error, Error, Reduce(27), Reduce(27), Error, Error, Error, Error, Error, Error, Error, Error, 
    Error, Error, Error, Reduce(39), Error, Error, Error, Error, Error, Error, Error, Error, Reduce(180), 
    Error, Error, Reduce(180), Error, Shift(194), Reduce(180), Shift(195), Reduce(35), Reduce(35), Reduce(35), 
    Reduce(27), Reduce(35), Error, Error, Error, Error, Error, Error, Error, Reduce(180), Reduce(180), 
    Reduce(180), Error, Reduce(180), Reduce(180), Reduce(35), Reduce(35), Error, Reduce(34), Reduce(34), 
    Reduce(34), Error, Reduce(34), Error, Error, Error, Error, Error, Error, Error, Error, Error, Error, 
    Error, Error, Error, Reduce(34), Reduce(34), Error, Error, Error, Error, Error, Error, Reduce(180), 
    Reduce(180), Error, Error, Reduce(35), Error, Error, Error, Error, Error, Error, Error, Error, Error, 
    Error, Error, Error, Error, Error, Error, Error, Error, Error, Reduce(40), Reduce(40), Reduce(34), 
    Reduce(33), Reduce(33), Reduce(33), Shift(78), Reduce(33), Reduce(180), Reduce(180), Error, Shift(79), 
    Error, Shift(80), Error, Shift(81), Shift(82), Error, Shift(83), Error, Error, Reduce(33), Reduce(33), 
    Error, Error, Error, Error, Error, Reduce(40), Reduce(40), Reduce(40), Reduce(40), Reduce(40), Reduce(40), 
    Reduce(40), Reduce(40), Reduce(40), Reduce(40), Reduce(40), Reduce(40), Reduce(40), Reduce(40), Reduce(40), 
    Reduce(40), Reduce(40), Reduce(40), Reduce(40), Error, Error, Error, Error, Reduce(32), Reduce(32), 
    Reduce(32), Reduce(33), Reduce(32), Error, Error, Error, Error, Error, Error, Error, Error, Error, 
    Error, Reduce(79), Error, Error, Reduce(32), Reduce(32), Error, Error, Error, Error, Error, Error, 
    Error, Error, Error, Reduce(39), Reduce(39), Error, Error, Error, Error, Error, Error, Error, Error, 
    Error, Error, Error, Error, Error, Error, Error, Error, Error, Shift(85), Reduce(27), Reduce(27), 
    Reduce(32), Error, Error, Error, Error, Error, Reduce(39), Reduce(39), Reduce(39), Reduce(39), Reduce(39), 
    Reduce(39), Reduce(39), Reduce(39), Reduce(39), Reduce(39), Reduce(39), Reduce(39), Reduce(39), Reduce(39), 
    Reduce(39), Reduce(39), Reduce(39), Reduce(39), Reduce(39), Error, Reduce(27), Reduce(27), Reduce(27), 
    Reduce(27), Reduce(27), Reduce(27), Reduce(27), Reduce(27), Reduce(27), Reduce(27), Reduce(27), Reduce(27), 
    Reduce(27), Reduce(27), Reduce(27), Reduce(27), Reduce(27), Reduce(27), Reduce(27), Error, Reduce(35), 
    Reduce(35), Error, Error, Reduce(180), Reduce(180), Reduce(180), Reduce(180), Reduce(180), Reduce(180), 
    Reduce(180), Reduce(180), Error, Error, Error, Error, Error, Error, Error, Error, Error, Reduce(34), 
    Reduce(34), Error, Error, Error, Error, Error, Reduce(35), Reduce(35), Reduce(35), Reduce(35), Reduce(35), 
    Reduce(35), Reduce(35), Reduce(35), Reduce(35), Reduce(35), Reduce(35), Reduce(35), Reduce(35), Reduce(35), 
    Reduce(35), Reduce(35), Reduce(35), Reduce(35), Reduce(35), Error, Error, Reduce(34), Reduce(34), 
    Reduce(34), Reduce(34), Reduce(34), Reduce(34), Reduce(34), Reduce(34), Reduce(34), Reduce(34), Reduce(34), 
    Reduce(34), Reduce(34), Reduce(34), Reduce(34), Reduce(34), Reduce(34), Reduce(34), Reduce(34), Error, 
    Error, Error, Error, Error, Reduce(33), Reduce(33), Reduce(31), Reduce(31), Reduce(31), Error, Reduce(31), 
    Error, Error, Error, Shift(86), Error, Shift(87), Shift(88), Shift(89), Shift(90), Shift(91), Error, 
    Shift(92), Shift(93), Reduce(31), Reduce(31), Error, Error, Error, Error, Error, Error, Reduce(33), 
    Reduce(33), Reduce(33), Reduce(33), Reduce(33), Reduce(33), Reduce(33), Reduce(33), Reduce(33), Reduce(33), 
    Reduce(33), Reduce(33), Reduce(33), Reduce(33), Reduce(33), Reduce(33), Reduce(33), Reduce(33), Reduce(33), 
    Error, Reduce(32), Reduce(32), Reduce(30), Reduce(30), Reduce(30), Reduce(31), Reduce(30), Error, 
    Error, Error, Error, Error, Error, Error, Error, Error, Error, Error, Error, Error, Reduce(30), Reduce(30), 
    Error, Error, Error, Error, Error, Error, Reduce(32), Reduce(32), Reduce(32), Reduce(32), Reduce(32), 
    Reduce(32), Reduce(32), Reduce(32), Reduce(32), Reduce(32), Reduce(32), Reduce(32), Reduce(32), Reduce(32), 
    Reduce(32), Reduce(32), Reduce(32), Reduce(32), Reduce(32), Error, Error, Error, Reduce(29), Reduce(29), 
    Reduce(29), Reduce(30), Reduce(29), Error, Error, Error, Error, Error, Error, Error, Error, Error, 
    Error, Error, Error, Error, Reduce(29), Reduce(29), Reduce(28), Reduce(28), Reduce(28), Error, Reduce(28), 
    Error, Error, Error, Error, Error, Error, Error, Error, Error, Error, Error, Error, Error, Reduce(28), 
    Reduce(28), Error, Error, Error, Error, Error, Error, Error, Error, Error, Error, Error, Reduce(29), 
    Error, Error, Error, Error, Error, Shift(191), Reduce(176), Error, Error, Reduce(176), Shift(192), 
    Reduce(176), Reduce(176), Reduce(176), Error, Shift(193), Reduce(38), Reduce(38), Reduce(38), Reduce(28), 
    Reduce(38), Error, Error, Error, Error, Error, Reduce(176), Reduce(176), Reduce(176), Error, Reduce(176), 
    Reduce(176), Error, Error, Reduce(38), Reduce(38), Reduce(20), Reduce(20), Reduce(20), Error, Reduce(20), 
    Error, Error, Error, Error, Error, Error, Error, Error, Error, Error, Error, Error, Error, Error, 
    Reduce(20), Error, Error, Error, Error, Error, Reduce(176), Reduce(176), Error, Error, Error, Error, 
    Reduce(38), Error, Error, Reduce(31), Reduce(31), Error, Error, Error, Error, Error, Error, Error, 
    Error, Error, Error, Error, Error, Error, Error, Error, Reduce(20), Error, Error, Error, Error, Reduce(176), 
    Reduce(176), Error, Error, Error, Error, Reduce(31), Reduce(31), Reduce(31), Reduce(31), Reduce(31), 
    Reduce(31), Reduce(31), Reduce(31), Reduce(31), Reduce(31), Reduce(31), Reduce(31), Reduce(31), Reduce(31), 
    Reduce(31), Reduce(31), Reduce(31), Reduce(31), Reduce(31), Error, Reduce(30), Reduce(30), Reduce(26), 
    Reduce(26), Reduce(26), Error, Reduce(26), Error, Error, Error, Error, Error, Error, Error, Error, 
    Error, Error, Error, Error, Error, Error, Reduce(26), Error, Error, Error, Error, Error, Error, Reduce(30), 
    Reduce(30), Reduce(30), Reduce(30), Reduce(30), Reduce(30), Reduce(30), Reduce(30), Reduce(30), Reduce(30), 
    Reduce(30), Reduce(30), Reduce(30), Reduce(30), Reduce(30), Reduce(30), Reduce(30), Reduce(30), Reduce(30), 
    Error, Reduce(29), Reduce(29), Error, Error, Error, Reduce(26), Error, Error, Error, Error, Error, 
    Error, Error, Error, Error, Error, Error, Error, Error, Error, Reduce(28), Reduce(28), Error, Error, 
    Error, Error, Error, Error, Reduce(29), Reduce(29), Reduce(29), Reduce(29), Reduce(29), Reduce(29), 
    Reduce(29), Reduce(29), Reduce(29), Reduce(29), Reduce(29), Reduce(29), Reduce(29), Reduce(29), Reduce(29), 
    Reduce(29), Reduce(29), Reduce(29), Reduce(29), Error, Reduce(28), Reduce(28), Reduce(28), Reduce(28), 
    Reduce(28), Reduce(28), Reduce(28), Reduce(28), Reduce(28), Reduce(28), Reduce(28), Reduce(28), Reduce(28), 
    Reduce(28), Reduce(28), Reduce(28), Reduce(28), Reduce(28), Reduce(28), Reduce(100), Reduce(38), 
    Reduce(38), Reduce(176), Reduce(176), Reduce(176), Reduce(176), Reduce(176), Reduce(176), Reduce(176), 
    Reduce(176), Error, Error, Error, Error, Error, Error, Error, Error, Error, Error, Reduce(20), Shift(2), 
    Error, Error, Error, Error, Error, Error, Reduce(38), Reduce(38), Reduce(38), Reduce(38), Reduce(38), 
    Reduce(38), Reduce(38), Reduce(38), Reduce(38), Reduce(38), Reduce(38), Reduce(38), Reduce(38), Reduce(38), 
    Reduce(38), Reduce(38), Reduce(38), Reduce(38), Reduce(38), Error, Shift(3), Shift(4), Shift(5), 
    Shift(6), Shift(7), Shift(8), Shift(9), Shift(10), Shift(11), Shift(12), Shift(13), Shift(14), Shift(15), 
    Shift(16), Shift(17), Shift(18), Shift(19), Shift(20), Shift(21), Reduce(25), Reduce(25), Reduce(25), 
    Error, Reduce(25), Shift(191), Reduce(175), Error, Error, Reduce(175), Shift(192), Reduce(175), Reduce(175), 
    Reduce(175), Error, Shift(193), Error, Error, Error, Reduce(25), Error, Error, Error, Error, Error, 
    Error, Reduce(175), Reduce(175), Reduce(175), Error, Reduce(175), Reduce(175), Error, Error, Error, 
    Error, Error, Reduce(26), Reduce(26), Error, Reduce(207), Error, Error, Error, Error, Reduce(207), 
    Error, Reduce(207), Error, Reduce(207), Reduce(207), Reduce(25), Reduce(207), Error, Error, Error, 
    Error, Error, Error, Error, Error, Reduce(175), Reduce(175), Error, Error, Reduce(26), Reduce(26), 
    Reduce(26), Reduce(26), Reduce(26), Reduce(26), Reduce(26), Reduce(26), Reduce(26), Reduce(26), Reduce(26), 
    Reduce(26), Reduce(26), Reduce(26), Reduce(26), Reduce(26), Reduce(26), Reduce(26), Reduce(26), Reduce(24), 
    Reduce(24), Reduce(24), Error, Reduce(24), Error, Error, Error, Reduce(175), Reduce(175), Error, 
    Error, Error, Error, Error, Error, Error, Error, Error, Reduce(24), Reduce(23), Reduce(23), Reduce(23), 
    Error, Reduce(23), Error, Error, Error, Error, Error, Error, Error, Error, Error, Error, Error, Error, 
    Error, Error, Reduce(23), Error, Error, Error, Error, Error, Error, Error, Error, Error, Reduce(207), 
    Error, Reduce(24), Error, Error, Error, Error, Error, Reduce(172), Reduce(172), Error, Error, Reduce(172), 
    Reduce(172), Reduce(172), Reduce(172), Reduce(172), Shift(2), Reduce(172), Reduce(22), Reduce(22), 
    Reduce(22), Reduce(23), Reduce(22), Error, Error, Error, Error, Error, Reduce(172), Reduce(172), 
    Reduce(172), Error, Reduce(172), Reduce(172), Error, Error, Error, Reduce(22), Reduce(66), Reduce(100), 
    Shift(1), Error, Error, Shift(3), Shift(4), Shift(5), Shift(6), Shift(7), Shift(8), Shift(9), Shift(10), 
    Shift(11), Shift(12), Shift(13), Shift(14), Shift(15), Shift(16), Shift(17), Shift(18), Shift(19), 
    Shift(20), Shift(21), Error, Reduce(172), Reduce(172), Error, Error, Error, Error, Reduce(22), Error, 
    Error, Error, Error, Error, Error, Error, Error, Error, Error, Error, Error, Error, Error, Reduce(25), 
    Reduce(25), Error, Error, Error, Reduce(66), Error, Error, Error, Error, Reduce(172), Reduce(172), 
    Error, Error, Error, Error, Error, Error, Reduce(175), Reduce(175), Reduce(175), Reduce(175), Reduce(175), 
    Reduce(175), Reduce(175), Reduce(175), Error, Error, Reduce(25), Reduce(25), Reduce(25), Reduce(25), 
    Reduce(25), Reduce(25), Reduce(25), Reduce(25), Reduce(25), Reduce(25), Reduce(25), Reduce(25), Reduce(25), 
    Reduce(25), Reduce(25), Reduce(25), Reduce(25), Reduce(25), Reduce(25), Reduce(207), Error, Reduce(207), 
    Reduce(207), Reduce(207), Reduce(207), Reduce(207), Error, Reduce(207), Reduce(207), Error, Error, 
    Error, Error, Error, Error, Error, Error, Error, Error, Error, Error, Error, Error, Error, Error, 
    Error, Error, Error, Error, Error, Error, Error, Error, Error, Error, Error, Reduce(24), Reduce(24), 
    Error, Error, Error, Error, Error, Error, Error, Error, Error, Error, Error, Error, Error, Error, 
    Error, Error, Error, Error, Reduce(23), Reduce(23), Error, Error, Error, Error, Error, Error, Reduce(24), 
    Reduce(24), Reduce(24), Reduce(24), Reduce(24), Reduce(24), Reduce(24), Reduce(24), Reduce(24), Reduce(24), 
    Reduce(24), Reduce(24), Reduce(24), Reduce(24), Reduce(24), Reduce(24), Reduce(24), Reduce(24), Reduce(24), 
    Error, Reduce(23), Reduce(23), Reduce(23), Reduce(23), Reduce(23), Reduce(23), Reduce(23), Reduce(23), 
    Reduce(23), Reduce(23), Reduce(23), Reduce(23), Reduce(23), Reduce(23), Reduce(23), Reduce(23), Reduce(23), 
    Reduce(23), Reduce(23), Error, Reduce(22), Reduce(22), Reduce(172), Reduce(172), Reduce(172), Reduce(172), 
    Reduce(172), Reduce(172), Reduce(172), Reduce(172), Error, Error, Error, Error, Error, Error, Error, 
    Error, Error, Error, Reduce(66), Shift(2), Error, Error, Error, Reduce(66), Reduce(100), Shift(1), 
    Reduce(22), Reduce(22), Reduce(22), Reduce(22), Reduce(22), Reduce(22), Reduce(22), Reduce(22), Reduce(22), 
    Reduce(22), Reduce(22), Reduce(22), Reduce(22), Reduce(22), Reduce(22), Reduce(22), Reduce(22), Reduce(22), 
    Reduce(22), Error, Shift(3), Shift(4), Shift(5), Shift(6), Shift(7), Shift(8), Shift(9), Shift(10), 
    Shift(11), Shift(12), Shift(13), Shift(14), Shift(15), Shift(16), Shift(17), Shift(18), Shift(19), 
    Shift(20), Shift(21), Reduce(171), Reduce(171), Error, Error, Reduce(171), Reduce(171), Reduce(171), 
    Reduce(171), Reduce(171), Reduce(66), Reduce(171), Error, Error, Error, Error, Error, Error, Error, 
    Error, Error, Error, Reduce(171), Reduce(171), Reduce(171), Error, Reduce(171), Reduce(171), Reduce(173), 
    Reduce(173), Error, Error, Reduce(173), Reduce(173), Reduce(173), Reduce(173), Reduce(173), Error, 
    Reduce(173), Error, Error, Error, Error, Error, Error, Error, Error, Error, Error, Reduce(173), Reduce(173), 
    Reduce(173), Reduce(4), Reduce(173), Reduce(173), Shift(191), Reduce(177), Reduce(171), Reduce(171), 
    Reduce(177), Shift(192), Reduce(177), Reduce(177), Reduce(177), Error, Shift(193), Error, Error, 
    Error, Error, Error, Error, Error, Error, Error, Error, Reduce(177), Reduce(177), Reduce(177), Error, 
    Reduce(177), Reduce(177), Error, Error, Reduce(173), Reduce(173), Error, Error, Reduce(171), Reduce(171), 
    Error, Error, Reduce(4), Error, Reduce(4), Error, Reduce(174), Reduce(174), Error, Reduce(5), Reduce(174), 
    Reduce(174), Reduce(174), Reduce(174), Reduce(174), Error, Reduce(174), Error, Error, Error, Error, 
    Reduce(177), Reduce(177), Error, Error, Reduce(173), Reduce(173), Reduce(174), Reduce(174), Reduce(174), 
    Accept(1), Reduce(174), Reduce(174), Error, Error, Error, Error, Error, Error, Error, Error, Error, 
    Error, Error, Error, Error, Error, Error, Error, Reduce(5), Error, Reduce(5), Reduce(177), Reduce(177), 
    Error, Error, Reduce(3), Error, Error, Error, Error, Error, Reduce(174), Reduce(174), Error, Error, 
    Error, Error, Error, Error, Reduce(1), Error, Reduce(1), Error, Error, Error, Error, Error, Error, 
    Error, Error, Error, Error, Error, Error, Error, Error, Error, Shift(2), Error, Error, Error, Error, 
    Reduce(174), Reduce(174), Error, Reduce(3), Error, Reduce(3), Error, Error, Error, Error, Error, 
    Error, Error, Error, Error, Error, Error, Reduce(2), Error, Error, Error, Error, Shift(3), Shift(4), 
    Shift(5), Shift(6), Shift(7), Shift(8), Shift(9), Shift(10), Shift(11), Shift(12), Shift(13), Shift(14), 
    Shift(15), Shift(16), Shift(17), Shift(18), Shift(19), Shift(20), Shift(21), Error, Error, Error, 
    Error, Error, Error, Error, Error, Reduce(171), Reduce(171), Reduce(171), Reduce(171), Reduce(171), 
    Reduce(171), Reduce(171), Reduce(171), Reduce(2), Error, Reduce(2), Error, Error, Error, Error, Error, 
    Error, Error, Error, Error, Error, Error, Accept(0), Error, Error, Error, Error, Reduce(173), Reduce(173), 
    Reduce(173), Reduce(173), Reduce(173), Reduce(173), Reduce(173), Reduce(173), Error, Error, Error, 
    Error, Error, Error, Error, Error, Error, Error, Error, Error, Error, Error, Error, Error, Error, 
    Error, Error, Reduce(177), Reduce(177), Reduce(177), Reduce(177), Reduce(177), Reduce(177), Reduce(177), 
    Reduce(177), Reduce(0), Error, Reduce(0), Error, Error, Error, Error, Error, Error, Error, Error, 
    Error, Error, Error, Error, Error, Reduce(4), Reduce(4), Error, Reduce(66), Error, Shift(1), Error, 
    Error, Error, Error, Error, Error, Error, Error, Error, Error, Error, Reduce(174), Reduce(174), Reduce(174), 
    Reduce(174), Reduce(174), Reduce(174), Reduce(174), Reduce(174), Error, Error, Error, Reduce(4), 
    Reduce(4), Reduce(4), Reduce(4), Reduce(4), Reduce(4), Reduce(4), Reduce(4), Reduce(4), Reduce(4), 
    Reduce(4), Reduce(4), Reduce(4), Reduce(4), Reduce(4), Reduce(4), Reduce(4), Reduce(4), Reduce(4), 
    Reduce(5), Reduce(5), Error, Error, Error, Error, Error, Error, Error, Error, Error, Error, Error, 
    Error, Error, Error, Error, Error, Error, Error, Error, Reduce(1), Reduce(1), Error, Error, Error, 
    Error, Error, Reduce(5), Reduce(5), Reduce(5), Reduce(5), Reduce(5), Reduce(5), Reduce(5), Reduce(5), 
    Reduce(5), Reduce(5), Reduce(5), Reduce(5), Reduce(5), Reduce(5), Reduce(5), Reduce(5), Reduce(5), 
    Reduce(5), Reduce(5), Reduce(3), Reduce(3), Reduce(1), Reduce(1), Reduce(1), Reduce(1), Reduce(1), 
    Reduce(1), Reduce(1), Reduce(1), Reduce(1), Reduce(1), Reduce(1), Reduce(1), Reduce(1), Reduce(1), 
    Reduce(1), Reduce(1), Reduce(1), Reduce(1), Reduce(1), Error, Error, Error, Error, Error, Error, 
    Error, Reduce(3), Reduce(3), Reduce(3), Reduce(3), Reduce(3), Reduce(3), Reduce(3), Reduce(3), Reduce(3), 
    Reduce(3), Reduce(3), Reduce(3), Reduce(3), Reduce(3), Reduce(3), Reduce(3), Reduce(3), Reduce(3), 
    Reduce(3), Error, Error, Error, Reduce(52), Reduce(52), Reduce(52), Error, Reduce(2), Reduce(2), 
    Error, Error, Error, Error, Error, Error, Error, Error, Error, Error, Error, Error, Reduce(52), Error, 
    Error, Error, Error, Error, Error, Error, Error, Error, Error, Error, Error, Error, Reduce(2), Reduce(2), 
    Reduce(2), Reduce(2), Reduce(2), Reduce(2), Reduce(2), Reduce(2), Reduce(2), Reduce(2), Reduce(2), 
    Reduce(2), Reduce(2), Reduce(2), Reduce(2), Reduce(2), Reduce(2), Reduce(2), Reduce(2), Reduce(52), 
    Error, Error, Error, Error, Error, Error, Reduce(0), Reduce(0), Error, Error, Error, Error, Error, 
    Error, Error, Error, Error, Error, Error, Error, Error, Error, Error, Error, Error, Reduce(66), Shift(2), 
    Error, Error, Error, Error, Error, Error, Error, Reduce(0), Reduce(0), Reduce(0), Reduce(0), Reduce(0), 
    Reduce(0), Reduce(0), Reduce(0), Reduce(0), Reduce(0), Reduce(0), Reduce(0), Reduce(0), Reduce(0), 
    Reduce(0), Reduce(0), Reduce(0), Reduce(0), Reduce(0), Shift(3), Shift(4), Shift(5), Shift(6), Shift(7), 
    Shift(8), Shift(9), Shift(10), Shift(11), Shift(12), Shift(13), Shift(14), Shift(15), Shift(16), 
    Shift(17), Shift(18), Shift(19), Shift(20), Shift(21), Reduce(183), Error, Error, Reduce(183), Error, 
    Error, Reduce(183), Error, Error, Error, Reduce(182), Error, Error, Reduce(182), Error, Error, Reduce(182), 
    Error, Error, Error, Reduce(183), Reduce(183), Reduce(183), Reduce(181), Reduce(183), Reduce(183), 
    Reduce(181), Error, Error, Reduce(181), Reduce(182), Reduce(182), Reduce(182), Error, Reduce(182), 
    Reduce(182), Error, Error, Error, Error, Error, Error, Error, Reduce(181), Reduce(181), Reduce(181), 
    Error, Reduce(181), Reduce(181), Error, Error, Error, Error, Error, Error, Reduce(183), Reduce(183), 
    Error, Error, Error, Error, Error, Error, Error, Error, Reduce(182), Reduce(182), Error, Reduce(185), 
    Error, Error, Reduce(185), Error, Error, Reduce(185), Error, Error, Error, Reduce(181), Reduce(181), 
    Error, Error, Error, Error, Error, Error, Reduce(183), Reduce(183), Reduce(185), Reduce(185), Reduce(185), 
    Error, Reduce(185), Reduce(185), Reduce(52), Shift(2), Reduce(182), Reduce(182), Error, Error, Error, 
    Error, Error, Error, Error, Error, Reduce(206), Error, Error, Reduce(181), Reduce(181), Reduce(206), 
    Error, Reduce(206), Error, Reduce(206), Reduce(206), Error, Reduce(206), Error, Error, Error, Error, 
    Reduce(185), Reduce(185), Error, Error, Shift(8), Shift(9), Shift(10), Shift(11), Shift(12), Shift(13), 
    Shift(14), Shift(15), Shift(16), Shift(17), Shift(18), Shift(19), Shift(20), Shift(21), Reduce(187), 
    Error, Error, Reduce(187), Error, Error, Reduce(187), Error, Error, Error, Error, Error, Error, Reduce(185), 
    Reduce(185), Error, Error, Error, Error, Error, Reduce(187), Reduce(187), Shift(198), Reduce(186), 
    Shift(199), Reduce(187), Reduce(186), Error, Error, Reduce(186), Error, Error, Error, Error, Error, 
    Error, Error, Error, Error, Error, Error, Error, Error, Reduce(186), Reduce(186), Shift(198), Error, 
    Shift(199), Reduce(186), Error, Error, Error, Error, Error, Reduce(205), Reduce(187), Reduce(187), 
    Error, Reduce(206), Reduce(205), Error, Reduce(205), Error, Reduce(205), Reduce(205), Error, Reduce(205), 
    Error, Error, Error, Error, Error, Error, Error, Error, Error, Error, Error, Reduce(186), Reduce(186), 
    Error, Error, Error, Error, Error, Error, Reduce(187), Reduce(187), Error, Shift(196), Shift(197), 
    Reduce(183), Reduce(183), Reduce(183), Reduce(183), Reduce(183), Reduce(183), Error, Error, Shift(196), 
    Shift(197), Reduce(182), Reduce(182), Reduce(182), Reduce(182), Reduce(182), Reduce(182), Error, 
    Error, Reduce(186), Reduce(186), Error, Shift(196), Shift(197), Reduce(181), Reduce(181), Reduce(181), 
    Reduce(181), Reduce(181), Reduce(181), Reduce(188), Error, Error, Reduce(188), Error, Error, Reduce(188), 
    Error, Error, Error, Error, Error, Error, Error, Error, Error, Error, Error, Error, Error, Reduce(188), 
    Reduce(188), Shift(198), Error, Shift(199), Reduce(188), Error, Reduce(205), Error, Error, Error, 
    Error, Error, Error, Error, Error, Error, Shift(196), Shift(197), Reduce(185), Reduce(185), Reduce(185), 
    Reduce(185), Reduce(185), Reduce(185), Error, Error, Error, Error, Error, Error, Error, Reduce(204), 
    Error, Error, Reduce(188), Reduce(188), Reduce(204), Error, Reduce(204), Error, Reduce(204), Reduce(204), 
    Error, Reduce(204), Error, Error, Error, Error, Error, Reduce(206), Error, Reduce(206), Reduce(206), 
    Reduce(206), Reduce(206), Reduce(206), Error, Reduce(206), Reduce(206), Error, Error, Error, Reduce(203), 
    Error, Error, Reduce(188), Reduce(188), Reduce(203), Error, Reduce(203), Error, Reduce(203), Reduce(203), 
    Shift(78), Reduce(203), Error, Error, Error, Shift(79), Error, Shift(80), Error, Shift(81), Shift(82), 
    Error, Shift(83), Error, Error, Error, Error, Error, Shift(200), Shift(201), Reduce(187), Reduce(187), 
    Reduce(187), Reduce(187), Shift(78), Error, Error, Error, Error, Shift(79), Error, Shift(168), Error, 
    Shift(81), Shift(82), Error, Shift(83), Error, Error, Error, Error, Shift(200), Shift(201), Reduce(186), 
    Reduce(186), Reduce(186), Reduce(186), Error, Error, Error, Error, Reduce(204), Shift(78), Error, 
    Error, Error, Error, Shift(79), Error, Shift(166), Error, Shift(81), Shift(82), Error, Shift(83), 
    Reduce(205), Error, Reduce(205), Reduce(205), Reduce(205), Reduce(205), Reduce(205), Error, Reduce(205), 
    Reduce(205), Error, Reduce(167), Error, Error, Error, Error, Reduce(167), Reduce(203), Reduce(167), 
    Error, Reduce(167), Reduce(167), Reduce(166), Reduce(167), Error, Error, Error, Reduce(166), Shift(85), 
    Reduce(166), Error, Reduce(166), Reduce(166), Error, Reduce(166), Reduce(165), Error, Error, Error, 
    Error, Reduce(165), Error, Reduce(165), Error, Reduce(165), Reduce(165), Reduce(164), Reduce(165), 
    Error, Error, Error, Reduce(164), Shift(85), Reduce(164), Error, Reduce(164), Reduce(164), Reduce(163), 
    Reduce(164), Error, Error, Error, Reduce(163), Error, Reduce(163), Error, Reduce(163), Reduce(163), 
    Error, Reduce(163), Error, Error, Error, Shift(200), Shift(201), Reduce(188), Reduce(188), Reduce(188), 
    Reduce(188), Reduce(168), Shift(85), Error, Error, Error, Reduce(168), Error, Reduce(168), Reduce(190), 
    Reduce(168), Reduce(168), Reduce(190), Reduce(168), Error, Reduce(190), Error, Error, Error, Error, 
    Error, Error, Error, Error, Error, Error, Reduce(167), Error, Error, Reduce(190), Reduce(190), Error, 
    Error, Error, Reduce(190), Error, Error, Reduce(166), Error, Error, Reduce(204), Error, Reduce(204), 
    Reduce(204), Reduce(204), Reduce(204), Reduce(204), Error, Reduce(204), Reduce(204), Reduce(165), 
    Error, Error, Error, Error, Error, Error, Error, Error, Error, Error, Reduce(164), Error, Error, 
    Reduce(190), Reduce(190), Error, Error, Error, Error, Error, Reduce(203), Reduce(163), Reduce(203), 
    Reduce(203), Reduce(203), Reduce(203), Reduce(203), Error, Reduce(203), Reduce(203), Error, Shift(86), 
    Error, Shift(87), Shift(88), Shift(89), Shift(90), Shift(91), Error, Shift(92), Shift(93), Error, 
    Error, Reduce(168), Reduce(190), Reduce(190), Error, Error, Error, Error, Error, Error, Error, Error, 
    Error, Shift(86), Error, Shift(87), Shift(88), Shift(89), Shift(90), Shift(91), Error, Shift(92), 
    Shift(93), Error, Error, Error, Error, Error, Error, Error, Error, Error, Error, Error, Error, Error, 
    Error, Error, Error, Error, Error, Shift(86), Error, Shift(87), Shift(88), Shift(89), Shift(90), 
    Shift(91), Error, Shift(92), Shift(93), Error, Error, Error, Error, Error, Error, Error, Error, Error, 
    Error, Error, Error, Error, Error, Reduce(167), Error, Reduce(167), Reduce(167), Reduce(167), Reduce(167), 
    Reduce(167), Error, Reduce(167), Reduce(167), Error, Reduce(166), Error, Reduce(166), Reduce(166), 
    Reduce(166), Reduce(166), Reduce(166), Error, Reduce(166), Reduce(166), Error, Error, Error, Reduce(165), 
    Error, Reduce(165), Reduce(165), Reduce(165), Reduce(165), Reduce(165), Error, Reduce(165), Reduce(165), 
    Error, Reduce(164), Error, Reduce(164), Reduce(164), Reduce(164), Reduce(164), Reduce(164), Error, 
    Reduce(164), Reduce(164), Error, Reduce(163), Error, Reduce(163), Reduce(163), Reduce(163), Reduce(163), 
    Reduce(163), Error, Reduce(163), Reduce(163), Error, Error, Shift(204), Error, Error, Reduce(192), 
    Error, Error, Reduce(192), Error, Error, Error, Reduce(168), Error, Reduce(168), Reduce(168), Reduce(168), 
    Reduce(168), Reduce(168), Error, Reduce(168), Reduce(168), Reduce(192), Reduce(192), Error, Error, 
    Error, Reduce(192), Error, Shift(202), Shift(203), Reduce(190), Reduce(190), Error, Error, Error, 
    Error, Error, Error, Error, Error, Error, Error, Error, Error, Error, Error, Error, Error, Error, 
    Error, Error, Error, Error, Error, Error, Error, Reduce(192), Reduce(192), Error, Error, Error, Error, 
    Error, Error, Error, Error, Error, Error, Error, Error, Error, Error, Error, Error, Error, Error, 
    Error, Error, Error, Error, Error, Error, Error, Error, Error, Error, Error, Reduce(192), Reduce(192), 
    Error, Error, Error, Error, Error, Error, Error, Error, Error, Error, Error, Error, Error, Error, 
    Error, Error, Error, Error, Error, Error, Error, Error, Error, Error, Error, Error, Error, Error, 
    Error, Error, Error, Error, Error, Error, Error, Error, Error, Error, Error, Error, Error, Error, 
    Error, Error, Error, Error, Error, Error, Error, Error, Error, Error, Error, Error, Error, Error, 
    Error, Error, Error, Error, Error, Error, Error, Error, Error, Error, Error, Error, Error, Error, 
    Error, Error, Error, Error, Error, Error, Error, Error, Error, Error, Error, Error, Error, Error, 
    Error, Error, Error, Error, Error, Error, Error, Error, Error, Error, Error, Error, Error, Error, 
    Error, Error, Error, Error, Error, Error, Error, Error, Error, Error, Error, Error, Error, Error, 
    Error, Error, Error, Error, Error, Error, Error, Error, Error, Error, Error, Error, Error, Error, 
    Error, Error, Error, Error, Error, Error, Error, Error, Error, Error, Error, Error, Error, Error, 
    Error, Error, Error, Error, Error, Error, Error, Error, Reduce(192), Reduce(192), 
];

static ACTION_CHECK: [Option<usize>; 8564] = [
    Some(191), None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, 
    None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, 
    Some(191), None, None, None, None, Some(191), None, Some(191), Some(225), Some(191), Some(191), Some(225), 
    Some(191), Some(225), Some(225), Some(225), Some(289), Some(46), Some(46), Some(46), Some(247), Some(46), 
    Some(250), Some(247), Some(216), Some(250), Some(191), Some(112), Some(225), Some(225), Some(225), 
    Some(272), Some(225), Some(225), Some(272), Some(259), Some(46), Some(205), Some(247), Some(250), 
    Some(250), Some(276), Some(258), Some(283), Some(276), Some(149), Some(149), Some(149), Some(272), 
    Some(272), Some(286), Some(286), Some(285), Some(285), Some(286), Some(281), Some(285), Some(258), 
    Some(103), Some(271), Some(112), Some(103), Some(271), Some(225), Some(225), Some(112), Some(282), 
    Some(112), Some(46), Some(112), Some(112), Some(277), Some(112), Some(216), Some(250), Some(103), 
    Some(103), Some(269), Some(32), Some(195), Some(103), Some(218), Some(268), Some(272), Some(218), 
    Some(64), Some(112), Some(45), Some(45), Some(45), Some(191), Some(45), Some(191), Some(191), Some(225), 
    Some(225), Some(149), Some(146), Some(218), Some(218), Some(64), Some(286), Some(64), Some(285), 
    Some(280), Some(280), Some(45), Some(267), Some(280), Some(266), Some(103), Some(146), Some(146), 
    Some(236), Some(254), Some(272), Some(259), Some(134), Some(205), Some(290), Some(134), Some(264), 
    Some(234), Some(258), Some(290), Some(234), Some(290), Some(236), Some(290), Some(290), Some(195), 
    Some(290), Some(246), Some(218), Some(134), Some(134), Some(264), Some(245), Some(45), Some(234), 
    Some(234), Some(110), Some(103), Some(110), Some(234), Some(290), Some(146), Some(55), Some(55), 
    Some(55), Some(112), Some(55), Some(112), Some(112), Some(212), Some(280), Some(279), Some(279), 
    Some(255), Some(110), Some(279), Some(255), Some(278), Some(278), Some(104), Some(218), Some(278), 
    Some(104), Some(210), Some(134), Some(211), Some(105), Some(252), Some(252), Some(234), Some(234), 
    Some(252), Some(182), Some(146), Some(263), Some(238), Some(104), Some(104), Some(210), Some(190), 
    Some(25), Some(105), Some(25), Some(52), Some(207), Some(52), Some(182), Some(182), Some(253), Some(263), 
    Some(238), Some(253), Some(186), Some(55), Some(190), Some(177), Some(177), Some(177), Some(183), 
    Some(25), Some(234), Some(234), Some(279), Some(151), Some(290), Some(288), Some(290), Some(290), 
    Some(278), Some(147), Some(288), Some(104), Some(288), Some(177), Some(288), Some(288), Some(244), 
    Some(288), Some(252), Some(244), Some(191), Some(191), Some(191), Some(191), Some(191), Some(191), 
    Some(191), Some(200), Some(191), Some(191), Some(46), Some(288), Some(160), Some(225), Some(225), 
    Some(225), Some(225), Some(225), Some(225), Some(225), Some(225), Some(159), Some(243), Some(104), 
    Some(210), Some(243), Some(177), Some(105), Some(191), Some(191), Some(191), Some(191), Some(191), 
    Some(191), Some(191), Some(191), Some(191), Some(191), Some(191), Some(191), Some(191), Some(191), 
    Some(191), Some(191), Some(191), Some(191), Some(191), Some(191), Some(191), Some(191), Some(191), 
    Some(191), Some(191), Some(191), Some(191), Some(191), Some(191), Some(191), Some(191), Some(42), 
    Some(112), Some(112), Some(112), Some(112), Some(112), Some(112), Some(112), Some(103), Some(112), 
    Some(112), Some(242), Some(32), Some(195), Some(242), Some(42), Some(288), Some(42), Some(288), Some(288), 
    Some(184), Some(45), Some(138), Some(138), Some(138), Some(214), Some(138), Some(206), Some(214), 
    Some(112), Some(112), Some(112), Some(112), Some(112), Some(112), Some(112), Some(112), Some(112), 
    Some(112), Some(112), Some(112), Some(112), Some(112), Some(112), Some(112), Some(112), Some(112), 
    Some(112), Some(112), Some(112), Some(112), Some(112), Some(112), Some(112), Some(112), Some(112), 
    Some(112), Some(112), Some(112), Some(112), Some(290), Some(290), Some(290), Some(290), Some(290), 
    Some(290), Some(290), Some(213), Some(290), Some(290), Some(213), Some(234), Some(234), Some(138), 
    Some(198), Some(110), Some(31), Some(198), Some(206), Some(42), Some(204), Some(55), Some(35), Some(35), 
    Some(35), Some(144), Some(35), Some(141), Some(290), Some(290), Some(290), Some(290), Some(290), 
    Some(290), Some(290), Some(290), Some(290), Some(290), Some(290), Some(290), Some(290), Some(290), 
    Some(290), Some(290), Some(290), Some(290), Some(290), Some(290), Some(290), Some(290), Some(290), 
    Some(290), Some(290), Some(290), Some(290), Some(290), Some(290), Some(290), Some(290), Some(25), 
    Some(287), Some(139), Some(52), Some(55), Some(55), Some(287), Some(122), Some(287), Some(181), Some(287), 
    Some(287), Some(60), Some(287), Some(35), Some(177), Some(197), Some(194), Some(150), Some(197), 
    Some(194), Some(150), Some(68), Some(63), Some(181), Some(153), Some(152), Some(287), Some(180), 
    Some(70), Some(121), Some(28), Some(288), Some(288), Some(288), Some(288), Some(288), Some(288), 
    Some(288), Some(68), Some(288), Some(288), Some(204), Some(180), Some(60), Some(60), Some(60), Some(60), 
    Some(60), Some(60), Some(60), Some(60), Some(60), Some(60), Some(60), Some(60), Some(60), Some(60), 
    Some(120), Some(119), Some(288), Some(288), Some(288), Some(288), Some(288), Some(288), Some(288), 
    Some(288), Some(288), Some(288), Some(288), Some(288), Some(288), Some(288), Some(288), Some(288), 
    Some(288), Some(288), Some(288), Some(288), Some(288), Some(288), Some(288), Some(288), Some(288), 
    Some(288), Some(288), Some(288), Some(288), Some(288), Some(288), Some(287), Some(275), Some(287), 
    Some(287), Some(100), Some(42), Some(275), Some(100), Some(275), Some(135), Some(275), Some(275), 
    Some(68), Some(275), Some(143), Some(153), Some(152), Some(143), Some(117), Some(70), Some(142), 
    Some(100), Some(100), Some(142), Some(135), Some(21), Some(100), Some(275), Some(239), Some(116), 
    Some(138), Some(115), Some(42), Some(42), Some(42), Some(42), Some(42), Some(42), Some(42), Some(42), 
    Some(42), Some(42), Some(42), Some(42), Some(42), Some(42), Some(42), Some(42), Some(42), Some(42), 
    Some(42), Some(140), Some(137), Some(69), Some(140), Some(137), Some(100), Some(100), Some(86), Some(114), 
    Some(274), Some(71), Some(41), Some(235), Some(40), Some(274), Some(235), Some(274), Some(63), Some(274), 
    Some(274), Some(65), Some(274), Some(86), Some(179), Some(28), Some(71), Some(41), Some(20), Some(40), 
    Some(235), Some(235), Some(57), Some(19), Some(31), Some(235), Some(274), Some(100), Some(100), Some(49), 
    Some(35), Some(275), Some(48), Some(275), Some(275), Some(63), Some(63), Some(63), Some(63), Some(63), 
    Some(63), Some(63), Some(63), Some(63), Some(63), Some(63), Some(63), Some(63), Some(63), Some(63), 
    Some(63), Some(63), Some(63), Some(63), Some(39), Some(235), Some(37), None, None, Some(273), None, 
    None, None, None, Some(273), None, Some(273), None, Some(273), Some(273), None, Some(273), Some(35), 
    Some(35), Some(69), Some(287), Some(287), Some(287), Some(287), Some(287), Some(287), Some(287), 
    None, Some(287), Some(287), Some(273), Some(235), Some(235), None, None, Some(274), None, Some(274), 
    Some(274), None, None, None, None, None, None, Some(21), None, Some(239), Some(287), Some(287), Some(287), 
    Some(287), Some(287), Some(287), Some(287), Some(287), Some(287), Some(287), Some(287), Some(287), 
    Some(287), Some(287), Some(287), Some(287), Some(287), Some(287), Some(287), Some(287), Some(287), 
    Some(287), Some(287), Some(287), Some(287), Some(287), Some(287), Some(287), Some(287), Some(287), 
    Some(287), Some(239), Some(239), Some(239), Some(239), Some(239), Some(239), Some(239), Some(239), 
    Some(239), Some(239), Some(239), Some(239), Some(239), Some(239), Some(179), Some(273), Some(265), 
    Some(273), Some(273), Some(20), None, Some(265), None, Some(265), Some(19), Some(265), Some(265), 
    None, Some(265), None, None, None, Some(275), Some(275), Some(275), Some(275), Some(275), Some(275), 
    Some(275), None, Some(275), Some(275), Some(265), Some(100), Some(100), None, Some(179), Some(179), 
    Some(179), Some(179), Some(179), Some(179), Some(179), Some(179), Some(179), Some(179), Some(179), 
    Some(179), Some(179), Some(179), Some(275), Some(275), Some(275), Some(275), Some(275), Some(275), 
    Some(275), Some(275), Some(275), Some(275), Some(275), Some(275), Some(275), Some(275), Some(275), 
    Some(275), Some(275), Some(275), Some(275), Some(275), Some(275), Some(275), Some(275), Some(275), 
    Some(275), Some(275), Some(275), Some(275), Some(275), Some(275), Some(275), Some(274), Some(274), 
    Some(274), Some(274), Some(274), Some(274), Some(274), None, Some(274), Some(274), None, Some(235), 
    Some(235), None, None, Some(265), None, Some(265), Some(265), None, None, Some(196), Some(196), None, 
    None, Some(196), None, None, Some(274), Some(274), Some(274), Some(274), Some(274), Some(274), Some(274), 
    Some(274), Some(274), Some(274), Some(274), Some(274), Some(274), Some(274), Some(274), Some(274), 
    Some(274), Some(274), Some(274), Some(274), Some(274), Some(274), Some(274), Some(274), Some(274), 
    Some(274), Some(274), Some(274), Some(274), Some(274), Some(274), Some(273), Some(273), Some(273), 
    Some(273), Some(273), Some(273), Some(273), None, Some(273), Some(273), Some(33), Some(33), Some(33), 
    Some(196), Some(33), None, None, None, None, None, None, None, Some(1), Some(1), Some(1), None, Some(1), 
    None, Some(273), Some(273), Some(273), Some(273), Some(273), Some(273), Some(273), Some(273), Some(273), 
    Some(273), Some(273), Some(273), Some(273), Some(273), Some(273), Some(273), Some(273), Some(273), 
    Some(273), Some(273), Some(273), Some(273), Some(273), Some(273), Some(273), Some(273), Some(273), 
    Some(273), Some(273), Some(273), Some(273), None, Some(249), Some(33), Some(44), Some(44), Some(44), 
    Some(249), Some(44), Some(249), None, Some(249), Some(249), None, Some(249), Some(1), None, None, 
    None, None, None, None, None, Some(44), Some(51), None, None, None, Some(249), None, None, None, 
    None, Some(265), Some(265), Some(265), Some(265), Some(265), Some(265), Some(265), None, Some(265), 
    Some(265), None, None, None, None, None, None, None, None, None, None, None, None, Some(44), None, 
    None, None, None, None, Some(265), Some(265), Some(265), Some(265), Some(265), Some(265), Some(265), 
    Some(265), Some(265), Some(265), Some(265), Some(265), Some(265), Some(265), Some(265), Some(265), 
    Some(265), Some(265), Some(265), Some(265), Some(265), Some(265), Some(265), Some(265), Some(265), 
    Some(265), Some(265), Some(265), Some(265), Some(265), Some(265), Some(249), Some(248), Some(249), 
    Some(249), Some(202), None, Some(248), None, Some(248), None, Some(248), Some(248), None, Some(248), 
    None, None, None, None, None, None, Some(56), Some(56), None, None, Some(56), None, None, Some(248), 
    Some(109), None, Some(196), Some(202), Some(202), Some(202), Some(202), Some(202), Some(202), Some(202), 
    Some(202), Some(202), Some(202), Some(202), Some(202), Some(202), Some(202), Some(202), Some(202), 
    Some(202), Some(202), Some(202), None, None, None, None, None, None, None, None, None, None, Some(241), 
    None, Some(202), Some(101), None, Some(241), Some(101), Some(241), Some(51), Some(241), Some(241), 
    Some(56), Some(241), None, Some(108), None, None, None, Some(33), None, Some(101), Some(101), None, 
    Some(34), Some(34), Some(101), Some(241), Some(34), None, None, Some(1), Some(248), None, Some(248), 
    Some(248), Some(51), Some(51), Some(51), Some(51), Some(51), Some(51), Some(51), Some(51), Some(51), 
    Some(51), Some(51), Some(51), Some(51), Some(51), Some(51), Some(51), Some(51), Some(51), Some(51), 
    None, Some(101), None, None, None, Some(240), Some(33), Some(33), None, None, Some(240), None, Some(240), 
    None, Some(240), Some(240), Some(44), Some(240), Some(1), Some(1), Some(34), Some(249), Some(249), 
    Some(249), Some(249), Some(249), Some(249), Some(249), None, Some(249), Some(249), Some(240), Some(101), 
    Some(101), None, None, Some(241), None, Some(241), Some(241), None, None, None, None, None, None, 
    None, None, Some(109), Some(249), Some(249), Some(249), Some(249), Some(249), Some(249), Some(249), 
    Some(249), Some(249), Some(249), Some(249), Some(249), Some(249), Some(249), Some(249), Some(249), 
    Some(249), Some(249), Some(249), Some(249), Some(249), Some(249), Some(249), Some(249), Some(249), 
    Some(249), Some(249), Some(249), Some(249), Some(249), Some(249), Some(109), Some(109), Some(109), 
    Some(109), Some(109), Some(109), Some(109), Some(109), Some(109), Some(109), Some(109), Some(109), 
    Some(109), Some(109), Some(108), Some(240), Some(193), Some(240), Some(240), None, None, Some(193), 
    None, Some(193), None, Some(193), Some(193), None, Some(193), None, None, None, Some(248), Some(248), 
    Some(248), Some(248), Some(248), Some(248), Some(248), None, Some(248), Some(248), Some(193), None, 
    Some(56), None, Some(108), Some(108), Some(108), Some(108), Some(108), Some(108), Some(108), Some(108), 
    Some(108), Some(108), Some(108), Some(108), Some(108), Some(108), Some(248), Some(248), Some(248), 
    Some(248), Some(248), Some(248), Some(248), Some(248), Some(248), Some(248), Some(248), Some(248), 
    Some(248), Some(248), Some(248), Some(248), Some(248), Some(248), Some(248), Some(248), Some(248), 
    Some(248), Some(248), Some(248), Some(248), Some(248), Some(248), Some(248), Some(248), Some(248), 
    Some(248), Some(241), Some(241), Some(241), Some(241), Some(241), Some(241), Some(241), None, Some(241), 
    Some(241), None, Some(101), Some(101), None, None, Some(193), Some(34), Some(193), Some(193), None, 
    None, None, None, None, None, None, None, None, Some(241), Some(241), Some(241), Some(241), Some(241), 
    Some(241), Some(241), Some(241), Some(241), Some(241), Some(241), Some(241), Some(241), Some(241), 
    Some(241), Some(241), Some(241), Some(241), Some(241), Some(241), Some(241), Some(241), Some(241), 
    Some(241), Some(241), Some(241), Some(241), Some(241), Some(241), Some(241), Some(241), Some(240), 
    Some(240), Some(240), Some(240), Some(240), Some(240), Some(240), None, Some(240), Some(240), None, 
    None, None, None, None, None, None, None, Some(50), None, None, None, None, None, None, None, None, 
    None, Some(240), Some(240), Some(240), Some(240), Some(240), Some(240), Some(240), Some(240), Some(240), 
    Some(240), Some(240), Some(240), Some(240), Some(240), Some(240), Some(240), Some(240), Some(240), 
    Some(240), Some(240), Some(240), Some(240), Some(240), Some(240), Some(240), Some(240), Some(240), 
    Some(240), Some(240), Some(240), Some(240), None, Some(189), None, Some(43), Some(43), Some(43), 
    Some(189), Some(43), Some(189), None, Some(189), Some(189), None, Some(189), None, None, None, None, 
    None, None, None, None, Some(43), Some(30), None, None, None, Some(189), None, None, None, None, 
    Some(193), Some(193), Some(193), Some(193), Some(193), Some(193), Some(193), None, Some(193), Some(193), 
    None, None, None, None, None, None, None, None, None, None, None, None, Some(43), None, None, None, 
    None, None, Some(193), Some(193), Some(193), Some(193), Some(193), Some(193), Some(193), Some(193), 
    Some(193), Some(193), Some(193), Some(193), Some(193), Some(193), Some(193), Some(193), Some(193), 
    Some(193), Some(193), Some(193), Some(193), Some(193), Some(193), Some(193), Some(193), Some(193), 
    Some(193), Some(193), Some(193), Some(193), Some(193), Some(189), Some(188), Some(189), Some(189), 
    Some(50), None, Some(188), None, Some(188), None, Some(188), Some(188), None, Some(188), None, None, 
    None, None, None, None, None, None, None, None, None, None, None, Some(188), None, None, None, Some(50), 
    Some(50), Some(50), Some(50), Some(50), Some(50), Some(50), Some(50), Some(50), Some(50), Some(50), 
    Some(50), Some(50), Some(50), Some(50), Some(50), Some(50), Some(50), Some(50), None, None, None, 
    None, None, None, None, None, None, None, Some(187), None, None, Some(237), None, Some(187), Some(237), 
    Some(187), Some(30), Some(187), Some(187), None, Some(187), None, Some(176), Some(176), Some(176), 
    None, None, None, Some(237), Some(237), None, None, None, Some(237), Some(187), None, None, None, 
    None, Some(188), Some(176), Some(188), Some(188), Some(30), Some(30), Some(30), Some(30), Some(30), 
    Some(30), Some(30), Some(30), Some(30), Some(30), Some(30), Some(30), Some(30), Some(30), Some(30), 
    Some(30), Some(30), Some(30), Some(30), None, Some(237), None, None, None, Some(133), None, None, 
    None, None, Some(133), Some(176), Some(133), None, Some(133), Some(133), Some(43), Some(133), None, 
    None, None, Some(189), Some(189), Some(189), Some(189), Some(189), Some(189), Some(189), None, Some(189), 
    Some(189), Some(133), None, Some(237), None, None, Some(187), None, Some(187), Some(187), None, None, 
    None, None, None, None, None, None, None, Some(189), Some(189), Some(189), Some(189), Some(189), 
    Some(189), Some(189), Some(189), Some(189), Some(189), Some(189), Some(189), Some(189), Some(189), 
    Some(189), Some(189), Some(189), Some(189), Some(189), Some(189), Some(189), Some(189), Some(189), 
    Some(189), Some(189), Some(189), Some(189), Some(189), Some(189), Some(189), Some(189), None, None, 
    None, None, None, None, Some(175), Some(175), Some(175), None, None, None, None, None, None, Some(133), 
    Some(132), Some(133), Some(133), None, None, Some(132), None, Some(132), Some(175), Some(132), Some(132), 
    None, Some(132), None, None, None, Some(188), Some(188), Some(188), Some(188), Some(188), Some(188), 
    Some(188), None, Some(188), Some(188), Some(132), None, None, None, None, None, None, None, None, 
    None, None, None, None, None, None, Some(175), None, None, Some(188), Some(188), Some(188), Some(188), 
    Some(188), Some(188), Some(188), Some(188), Some(188), Some(188), Some(188), Some(188), Some(188), 
    Some(188), Some(188), Some(188), Some(188), Some(188), Some(188), Some(188), Some(188), Some(188), 
    Some(188), Some(188), Some(188), Some(188), Some(188), Some(188), Some(188), Some(188), Some(188), 
    Some(187), Some(187), Some(187), Some(187), Some(187), Some(187), Some(187), Some(176), Some(187), 
    Some(187), None, Some(237), Some(237), None, None, Some(132), None, Some(132), Some(132), None, None, 
    None, None, None, None, None, None, None, Some(187), Some(187), Some(187), Some(187), Some(187), 
    Some(187), Some(187), Some(187), Some(187), Some(187), Some(187), Some(187), Some(187), Some(187), 
    Some(187), Some(187), Some(187), Some(187), Some(187), Some(187), Some(187), Some(187), Some(187), 
    Some(187), Some(187), Some(187), Some(187), Some(187), Some(187), Some(187), Some(187), Some(133), 
    Some(133), Some(133), Some(133), Some(133), Some(133), Some(133), None, Some(133), Some(133), None, 
    None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, 
    Some(133), Some(133), Some(133), Some(133), Some(133), Some(133), Some(133), Some(133), Some(133), 
    Some(133), Some(133), Some(133), Some(133), Some(133), Some(133), Some(133), Some(133), Some(133), 
    Some(133), Some(133), Some(133), Some(133), Some(133), Some(133), Some(133), Some(133), Some(133), 
    Some(133), Some(133), Some(133), Some(133), None, Some(131), None, None, None, None, Some(131), None, 
    Some(131), None, Some(131), Some(131), None, Some(131), None, Some(175), None, None, None, None, 
    None, None, None, None, None, None, None, Some(131), None, None, None, None, Some(132), Some(132), 
    Some(132), Some(132), Some(132), Some(132), Some(132), None, Some(132), Some(132), None, None, None, 
    None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, Some(132), 
    Some(132), Some(132), Some(132), Some(132), Some(132), Some(132), Some(132), Some(132), Some(132), 
    Some(132), Some(132), Some(132), Some(132), Some(132), Some(132), Some(132), Some(132), Some(132), 
    Some(132), Some(132), Some(132), Some(132), Some(132), Some(132), Some(132), Some(132), Some(132), 
    Some(132), Some(132), Some(132), Some(131), Some(130), Some(131), Some(131), None, None, Some(130), 
    None, Some(130), None, Some(130), Some(130), Some(209), Some(130), None, None, None, Some(209), None, 
    Some(209), None, Some(209), Some(209), None, Some(209), None, None, Some(130), None, None, None, 
    None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, 
    None, None, None, None, None, None, None, None, None, None, None, None, Some(127), None, None, Some(102), 
    None, Some(127), Some(102), Some(127), None, Some(127), Some(127), None, Some(127), None, None, None, 
    None, None, None, None, Some(102), Some(102), None, None, None, Some(102), Some(127), None, None, 
    None, None, Some(130), None, Some(130), Some(130), None, None, None, None, None, None, None, Some(209), 
    None, Some(209), Some(209), None, None, None, None, None, None, None, None, None, Some(102), None, 
    None, None, Some(126), None, None, None, None, Some(126), None, Some(126), None, Some(126), Some(126), 
    None, Some(126), None, None, None, Some(131), Some(131), Some(131), Some(131), Some(131), Some(131), 
    Some(131), None, Some(131), Some(131), Some(126), None, Some(102), None, None, Some(127), None, Some(127), 
    Some(127), None, None, None, None, None, None, None, None, None, Some(131), Some(131), Some(131), 
    Some(131), Some(131), Some(131), Some(131), Some(131), Some(131), Some(131), Some(131), Some(131), 
    Some(131), Some(131), Some(131), Some(131), Some(131), Some(131), Some(131), Some(131), Some(131), 
    Some(131), Some(131), Some(131), Some(131), Some(131), Some(131), Some(131), Some(131), Some(131), 
    Some(131), None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, 
    Some(126), Some(111), Some(126), Some(126), None, None, Some(111), None, Some(111), None, Some(111), 
    Some(111), None, Some(111), None, None, None, Some(130), Some(130), Some(130), Some(130), Some(130), 
    Some(130), Some(130), None, Some(130), Some(130), Some(111), Some(209), None, Some(209), Some(209), 
    Some(209), Some(209), Some(209), None, Some(209), Some(209), None, None, None, None, None, None, 
    None, Some(130), Some(130), Some(130), Some(130), Some(130), Some(130), Some(130), Some(130), Some(130), 
    Some(130), Some(130), Some(130), Some(130), Some(130), Some(130), Some(130), Some(130), Some(130), 
    Some(130), Some(130), Some(130), Some(130), Some(130), Some(130), Some(130), Some(130), Some(130), 
    Some(130), Some(130), Some(130), Some(130), Some(127), Some(127), Some(127), Some(127), Some(127), 
    Some(127), Some(127), None, Some(127), Some(127), None, Some(102), Some(102), None, None, Some(111), 
    None, Some(111), Some(111), None, None, None, None, None, None, None, None, None, Some(127), Some(127), 
    Some(127), Some(127), Some(127), Some(127), Some(127), Some(127), Some(127), Some(127), Some(127), 
    Some(127), Some(127), Some(127), Some(127), Some(127), Some(127), Some(127), Some(127), Some(127), 
    Some(127), Some(127), Some(127), Some(127), Some(127), Some(127), Some(127), Some(127), Some(127), 
    Some(127), Some(127), Some(126), Some(126), Some(126), Some(126), Some(126), Some(126), Some(126), 
    None, Some(126), Some(126), None, None, None, None, None, None, None, None, None, None, None, None, 
    None, None, None, None, None, Some(58), Some(126), Some(126), Some(126), Some(126), Some(126), Some(126), 
    Some(126), Some(126), Some(126), Some(126), Some(126), Some(126), Some(126), Some(126), Some(126), 
    Some(126), Some(126), Some(126), Some(126), Some(126), Some(126), Some(126), Some(126), Some(126), 
    Some(126), Some(126), Some(126), Some(126), Some(126), Some(126), Some(126), None, Some(58), None, 
    None, None, None, Some(58), None, Some(58), None, Some(58), Some(58), None, Some(58), None, None, 
    None, None, None, None, None, None, None, None, None, None, None, Some(58), None, None, None, None, 
    Some(111), Some(111), Some(111), Some(111), Some(111), Some(111), Some(111), None, Some(111), Some(111), 
    None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, 
    None, Some(111), Some(111), Some(111), Some(111), Some(111), Some(111), Some(111), Some(111), Some(111), 
    Some(111), Some(111), Some(111), Some(111), Some(111), Some(111), Some(111), Some(111), Some(111), 
    Some(111), Some(111), Some(111), Some(111), Some(111), Some(111), Some(111), Some(111), Some(111), 
    Some(111), Some(111), Some(111), Some(111), Some(58), Some(192), Some(58), Some(58), None, None, 
    Some(192), None, Some(192), None, Some(192), Some(192), Some(284), Some(192), None, None, None, Some(284), 
    None, Some(284), Some(284), Some(284), Some(284), None, Some(284), None, None, Some(192), None, None, 
    None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, 
    None, None, None, None, None, None, None, None, None, None, None, None, None, Some(129), None, None, 
    None, None, Some(129), None, Some(129), None, Some(129), Some(129), None, Some(129), None, None, 
    None, None, None, None, None, None, None, None, None, None, None, Some(129), None, None, None, None, 
    Some(192), None, Some(192), Some(192), None, None, None, None, None, None, None, None, None, None, 
    Some(284), None, None, None, None, None, None, None, None, None, None, None, None, None, Some(128), 
    None, None, None, None, Some(128), None, Some(128), None, Some(128), Some(128), None, Some(128), 
    None, None, None, Some(58), Some(58), Some(58), Some(58), Some(58), Some(58), Some(58), None, Some(58), 
    Some(58), Some(128), None, None, None, None, Some(129), None, Some(129), Some(129), None, None, None, 
    None, None, None, None, None, None, Some(58), Some(58), Some(58), Some(58), Some(58), Some(58), Some(58), 
    Some(58), Some(58), Some(58), Some(58), Some(58), Some(58), Some(58), Some(58), Some(58), Some(58), 
    Some(58), Some(58), Some(58), Some(58), Some(58), None, Some(58), Some(58), Some(58), Some(58), Some(58), 
    Some(58), Some(58), Some(58), None, None, None, None, None, None, None, None, None, None, None, None, 
    None, None, None, Some(128), Some(125), Some(128), Some(128), None, None, Some(125), None, Some(125), 
    None, Some(125), Some(125), None, Some(125), None, None, None, Some(192), Some(192), Some(192), Some(192), 
    Some(192), Some(192), Some(192), None, Some(192), Some(192), Some(125), Some(284), None, Some(284), 
    Some(284), Some(284), Some(284), Some(284), None, Some(284), Some(284), None, None, None, None, None, 
    None, None, Some(192), Some(192), Some(192), Some(192), Some(192), Some(192), Some(192), Some(192), 
    Some(192), Some(192), Some(192), Some(192), Some(192), Some(192), Some(192), Some(192), Some(192), 
    Some(192), Some(192), Some(192), Some(192), Some(192), None, Some(192), Some(192), Some(192), Some(192), 
    Some(192), Some(192), Some(192), Some(192), Some(129), Some(129), Some(129), Some(129), Some(129), 
    Some(129), Some(129), None, Some(129), Some(129), None, None, None, None, None, Some(125), None, 
    Some(125), Some(125), None, None, None, None, None, None, None, None, None, Some(129), Some(129), 
    Some(129), Some(129), Some(129), Some(129), Some(129), Some(129), Some(129), Some(129), Some(129), 
    Some(129), Some(129), Some(129), Some(129), Some(129), Some(129), Some(129), Some(129), Some(129), 
    Some(129), Some(129), None, Some(129), Some(129), Some(129), Some(129), Some(129), Some(129), Some(129), 
    Some(129), Some(128), Some(128), Some(128), Some(128), Some(128), Some(128), Some(128), None, Some(128), 
    Some(128), None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, 
    None, None, None, Some(128), Some(128), Some(128), Some(128), Some(128), Some(128), Some(128), Some(128), 
    Some(128), Some(128), Some(128), Some(128), Some(128), Some(128), Some(128), Some(128), Some(128), 
    Some(128), Some(128), Some(128), Some(128), Some(128), None, Some(128), Some(128), Some(128), Some(128), 
    Some(128), Some(128), Some(128), Some(128), None, Some(124), None, None, None, None, Some(124), None, 
    Some(124), None, Some(124), Some(124), None, Some(124), None, None, Some(118), None, None, None, 
    None, Some(118), None, Some(118), None, Some(118), Some(118), Some(124), Some(118), None, None, None, 
    Some(125), Some(125), Some(125), Some(125), Some(125), Some(125), Some(125), None, Some(125), Some(125), 
    Some(118), None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, 
    None, None, Some(125), Some(125), Some(125), Some(125), Some(125), Some(125), Some(125), Some(125), 
    Some(125), Some(125), Some(125), Some(125), Some(125), Some(125), Some(125), Some(125), Some(125), 
    Some(125), Some(125), Some(125), Some(125), Some(125), None, Some(125), Some(125), Some(125), Some(125), 
    Some(125), Some(125), Some(125), Some(125), Some(124), Some(61), Some(124), Some(124), None, None, 
    Some(61), None, Some(61), None, Some(61), Some(61), None, Some(61), None, Some(118), Some(224), None, 
    Some(118), Some(224), None, Some(224), Some(224), Some(224), None, None, None, Some(61), None, None, 
    None, None, None, None, None, None, Some(224), Some(224), Some(224), None, Some(224), Some(224), 
    None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, 
    Some(157), Some(157), None, Some(157), Some(157), Some(157), Some(157), Some(157), Some(157), Some(157), 
    Some(157), None, Some(224), Some(224), None, None, None, None, None, None, None, Some(157), Some(157), 
    Some(157), Some(157), Some(157), Some(157), None, None, None, None, None, Some(61), None, Some(61), 
    Some(61), None, None, None, None, None, None, None, Some(224), Some(224), None, None, None, None, 
    None, None, None, None, None, Some(157), None, Some(157), Some(157), None, None, None, None, Some(158), 
    None, None, None, None, Some(158), None, Some(158), Some(158), Some(158), Some(158), None, Some(158), 
    None, Some(124), Some(124), Some(124), Some(124), Some(124), Some(124), Some(124), None, Some(124), 
    Some(124), None, Some(157), Some(157), None, None, Some(118), None, Some(118), Some(118), Some(118), 
    Some(118), Some(118), None, Some(118), Some(118), None, None, None, Some(124), Some(124), Some(124), 
    Some(124), Some(124), Some(124), Some(124), Some(124), Some(124), Some(124), Some(124), Some(124), 
    Some(124), Some(124), Some(124), Some(124), Some(124), Some(124), Some(124), Some(124), Some(124), 
    Some(124), None, Some(124), Some(124), Some(124), Some(124), Some(124), Some(124), Some(124), Some(124), 
    None, None, None, Some(118), Some(118), Some(118), None, Some(118), Some(118), Some(118), Some(118), 
    Some(118), Some(118), Some(118), Some(118), None, None, None, None, None, Some(158), None, None, 
    None, None, None, None, None, None, None, None, None, Some(61), Some(61), Some(61), Some(61), Some(61), 
    Some(61), Some(61), None, Some(61), Some(61), None, None, None, None, None, None, None, None, None, 
    None, Some(224), Some(224), Some(224), Some(224), Some(224), Some(224), Some(224), Some(224), Some(61), 
    Some(61), Some(61), Some(61), Some(61), Some(61), Some(61), Some(61), Some(61), Some(61), Some(61), 
    Some(61), Some(61), Some(61), Some(61), Some(61), Some(61), Some(61), Some(61), Some(61), Some(61), 
    Some(61), None, Some(61), Some(61), Some(61), Some(61), Some(61), Some(61), Some(61), Some(61), Some(157), 
    None, Some(157), Some(157), Some(157), Some(157), Some(157), Some(157), Some(157), Some(157), Some(157), 
    Some(157), Some(157), Some(157), Some(157), Some(157), Some(157), Some(157), Some(157), Some(157), 
    Some(157), Some(157), Some(157), Some(89), Some(89), None, Some(89), Some(89), Some(89), Some(89), 
    Some(89), Some(89), Some(89), Some(89), None, None, None, None, None, None, None, None, None, None, 
    Some(89), Some(89), Some(89), Some(89), Some(89), Some(89), Some(83), Some(83), None, Some(83), Some(83), 
    Some(83), Some(83), Some(83), Some(83), Some(83), Some(83), Some(158), None, Some(158), Some(158), 
    Some(158), Some(158), Some(158), None, Some(158), Some(158), Some(83), Some(83), Some(83), Some(83), 
    Some(83), Some(83), Some(89), Some(173), Some(89), Some(89), None, None, Some(173), None, Some(173), 
    None, Some(173), Some(173), None, Some(173), Some(262), Some(262), None, Some(262), Some(262), Some(262), 
    Some(262), Some(262), Some(262), Some(262), Some(262), None, None, Some(83), None, Some(83), Some(83), 
    None, None, Some(89), Some(89), Some(262), Some(262), Some(262), Some(262), Some(262), Some(262), 
    Some(261), Some(261), None, Some(261), Some(261), Some(261), Some(261), Some(261), Some(261), Some(261), 
    Some(261), None, None, None, None, None, None, None, None, Some(83), Some(83), Some(261), Some(261), 
    Some(261), Some(261), Some(261), Some(261), Some(262), Some(59), Some(262), Some(262), None, None, 
    Some(59), None, Some(59), None, Some(59), Some(59), None, Some(59), Some(217), Some(217), None, Some(217), 
    Some(217), Some(217), Some(217), Some(217), Some(217), Some(217), Some(217), None, Some(173), Some(261), 
    None, Some(261), Some(261), None, None, Some(262), Some(262), Some(217), Some(217), Some(217), Some(217), 
    Some(217), Some(217), Some(215), Some(215), None, Some(215), Some(215), Some(215), Some(215), Some(215), 
    Some(215), Some(215), Some(215), None, None, None, None, None, None, None, None, Some(261), Some(261), 
    Some(215), Some(215), Some(215), Some(215), Some(215), Some(215), Some(217), None, Some(217), Some(217), 
    None, None, None, None, Some(172), None, None, None, None, Some(172), None, Some(172), None, Some(172), 
    Some(172), None, Some(172), None, None, Some(59), None, None, Some(59), Some(215), None, Some(215), 
    Some(215), None, None, Some(217), Some(217), None, Some(89), None, Some(89), Some(89), Some(89), 
    Some(89), Some(89), Some(89), Some(89), Some(89), Some(89), Some(89), Some(89), Some(89), Some(89), 
    Some(89), Some(89), Some(89), Some(89), Some(89), Some(89), Some(89), Some(89), None, Some(215), 
    Some(215), None, Some(83), None, Some(83), Some(83), Some(83), Some(83), Some(83), Some(83), Some(83), 
    Some(83), Some(83), Some(83), Some(83), Some(83), Some(83), Some(83), Some(83), Some(83), Some(83), 
    Some(83), Some(83), Some(83), Some(83), None, None, None, None, Some(173), None, Some(173), Some(173), 
    Some(173), Some(173), Some(173), None, Some(173), Some(173), None, Some(172), None, None, None, None, 
    Some(262), Some(262), Some(262), Some(262), Some(262), Some(262), Some(262), Some(262), Some(262), 
    Some(262), Some(262), Some(262), Some(262), Some(262), Some(262), Some(262), Some(262), Some(262), 
    Some(262), Some(262), Some(262), None, None, None, None, None, None, Some(261), Some(261), Some(261), 
    Some(261), Some(261), Some(261), Some(261), Some(261), Some(261), Some(261), Some(261), Some(261), 
    Some(261), Some(261), Some(261), Some(261), Some(261), Some(261), Some(261), Some(261), Some(261), 
    None, None, None, None, Some(59), None, Some(59), Some(59), Some(59), Some(59), Some(59), None, Some(59), 
    Some(59), None, None, None, None, None, None, Some(217), Some(217), Some(217), Some(217), Some(217), 
    Some(217), Some(217), Some(217), Some(217), Some(217), Some(217), Some(217), Some(217), Some(217), 
    Some(217), Some(217), Some(217), Some(217), Some(217), Some(217), Some(217), None, None, None, None, 
    None, None, Some(215), Some(215), Some(215), Some(215), Some(215), Some(215), Some(215), Some(215), 
    Some(215), Some(215), Some(215), Some(215), Some(215), Some(215), Some(215), Some(215), Some(215), 
    Some(215), Some(215), Some(215), Some(215), Some(208), Some(208), None, Some(208), Some(208), Some(208), 
    Some(208), Some(208), Some(208), Some(208), Some(208), Some(172), None, Some(172), Some(172), Some(172), 
    Some(172), Some(172), None, Some(172), Some(172), Some(208), Some(208), Some(208), Some(208), Some(208), 
    Some(208), Some(162), Some(162), None, Some(162), Some(162), Some(162), Some(162), Some(162), Some(162), 
    Some(162), Some(162), None, None, None, None, None, None, None, None, None, None, Some(162), Some(162), 
    Some(162), Some(162), Some(162), Some(162), Some(208), Some(171), Some(208), Some(208), None, None, 
    Some(171), None, Some(171), None, Some(171), Some(171), None, Some(171), Some(161), Some(161), None, 
    Some(161), Some(161), Some(161), Some(161), Some(161), Some(161), Some(161), Some(161), None, None, 
    Some(162), None, Some(162), Some(162), None, None, Some(208), Some(208), Some(161), Some(161), Some(161), 
    Some(161), Some(161), Some(161), Some(90), Some(90), None, Some(90), Some(90), Some(90), Some(90), 
    Some(90), Some(90), Some(90), Some(90), None, None, None, None, None, None, None, None, Some(162), 
    Some(162), Some(90), Some(90), Some(90), Some(90), Some(90), Some(90), Some(161), Some(170), Some(161), 
    Some(161), None, None, Some(170), None, Some(170), None, Some(170), Some(170), None, Some(170), Some(88), 
    Some(88), None, Some(88), Some(88), Some(88), Some(88), Some(88), Some(88), Some(88), Some(88), None, 
    Some(171), Some(90), None, Some(90), Some(90), None, None, Some(161), Some(161), Some(88), Some(88), 
    Some(88), Some(88), Some(88), Some(88), Some(87), Some(87), None, Some(87), Some(87), Some(87), Some(87), 
    Some(87), Some(87), Some(87), Some(87), None, None, None, None, None, None, None, None, Some(90), 
    Some(90), Some(87), Some(87), Some(87), Some(87), Some(87), Some(87), Some(88), None, Some(88), Some(88), 
    None, None, None, None, Some(169), None, None, None, None, Some(169), None, Some(169), None, Some(169), 
    Some(169), None, Some(169), None, None, None, None, None, Some(170), Some(87), None, Some(87), Some(87), 
    None, None, Some(88), Some(88), None, None, None, Some(208), Some(208), Some(208), Some(208), Some(208), 
    Some(208), Some(208), Some(208), Some(208), Some(208), Some(208), Some(208), Some(208), Some(208), 
    Some(208), Some(208), Some(208), Some(208), Some(208), Some(208), Some(208), None, Some(87), Some(87), 
    None, None, None, Some(162), Some(162), Some(162), Some(162), Some(162), Some(162), Some(162), Some(162), 
    Some(162), Some(162), Some(162), Some(162), Some(162), Some(162), Some(162), Some(162), Some(162), 
    Some(162), Some(162), Some(162), Some(162), None, None, None, None, Some(171), None, Some(171), Some(171), 
    Some(171), Some(171), Some(171), None, Some(171), Some(171), None, Some(169), None, None, None, None, 
    Some(161), Some(161), Some(161), Some(161), Some(161), Some(161), Some(161), Some(161), Some(161), 
    Some(161), Some(161), Some(161), Some(161), Some(161), Some(161), Some(161), Some(161), Some(161), 
    Some(161), Some(161), Some(161), None, None, None, None, None, None, Some(90), Some(90), Some(90), 
    Some(90), Some(90), Some(90), Some(90), Some(90), Some(90), Some(90), Some(90), Some(90), Some(90), 
    Some(90), Some(90), Some(90), Some(90), Some(90), Some(90), Some(90), Some(90), None, None, None, 
    None, Some(170), None, Some(170), Some(170), Some(170), Some(170), Some(170), None, Some(170), Some(170), 
    None, None, None, None, None, None, Some(88), Some(88), Some(88), Some(88), Some(88), Some(88), Some(88), 
    Some(88), Some(88), Some(88), Some(88), Some(88), Some(88), Some(88), Some(88), Some(88), Some(88), 
    Some(88), Some(88), Some(88), Some(88), None, None, None, None, None, None, Some(87), Some(87), Some(87), 
    Some(87), Some(87), Some(87), Some(87), Some(87), Some(87), Some(87), Some(87), Some(87), Some(87), 
    Some(87), Some(87), Some(87), Some(87), Some(87), Some(87), Some(87), Some(87), Some(82), Some(82), 
    None, Some(82), Some(82), Some(82), Some(82), Some(82), Some(82), Some(82), Some(82), Some(169), 
    None, Some(169), Some(169), Some(169), Some(169), Some(169), None, Some(169), Some(169), Some(82), 
    Some(82), Some(82), Some(82), Some(82), Some(82), Some(81), Some(81), None, Some(81), Some(81), Some(81), 
    Some(81), Some(81), Some(81), Some(81), Some(81), None, None, None, None, None, None, None, None, 
    None, None, Some(81), Some(81), Some(81), Some(81), Some(81), Some(81), Some(82), None, Some(82), 
    Some(82), None, None, None, None, None, None, None, None, None, None, Some(80), Some(80), None, Some(80), 
    Some(80), Some(80), Some(80), Some(80), Some(80), Some(80), Some(80), None, None, Some(81), None, 
    Some(81), Some(81), None, None, Some(82), Some(82), Some(80), Some(80), Some(80), Some(80), Some(80), 
    Some(80), Some(79), Some(79), None, Some(79), Some(79), Some(79), Some(79), Some(79), Some(79), Some(79), 
    Some(79), None, None, None, None, None, None, None, None, Some(81), Some(81), Some(79), Some(79), 
    Some(79), Some(79), Some(79), Some(79), Some(80), Some(168), Some(80), Some(80), None, None, Some(168), 
    None, Some(168), None, Some(168), Some(168), None, Some(168), Some(113), Some(113), None, Some(113), 
    None, Some(113), Some(113), Some(113), Some(113), Some(113), Some(113), None, None, Some(79), None, 
    Some(79), Some(79), None, None, Some(80), Some(80), Some(113), Some(113), Some(113), Some(113), Some(113), 
    Some(113), Some(260), Some(260), None, None, Some(260), Some(260), Some(260), Some(260), Some(260), 
    None, Some(260), None, None, None, None, None, None, None, None, Some(79), Some(79), Some(260), Some(260), 
    Some(260), Some(260), Some(260), Some(260), Some(113), None, None, Some(113), None, None, Some(185), 
    None, None, None, None, Some(185), None, Some(185), None, Some(185), Some(185), None, Some(185), 
    None, None, None, None, None, None, None, Some(168), None, None, Some(260), Some(260), None, Some(185), 
    Some(113), None, None, None, None, Some(82), Some(82), Some(82), Some(82), Some(82), Some(82), Some(82), 
    Some(82), Some(82), Some(82), Some(82), Some(82), Some(82), Some(82), Some(82), Some(82), Some(82), 
    Some(82), Some(82), Some(82), Some(82), None, Some(260), Some(260), None, None, None, Some(81), Some(81), 
    Some(81), Some(81), Some(81), Some(81), Some(81), Some(81), Some(81), Some(81), Some(81), Some(81), 
    Some(81), Some(81), Some(81), Some(81), Some(81), Some(81), Some(81), Some(81), Some(81), None, None, 
    None, None, None, None, None, None, None, None, None, None, None, Some(185), None, None, None, None, 
    None, None, Some(80), Some(80), Some(80), Some(80), Some(80), Some(80), Some(80), Some(80), Some(80), 
    Some(80), Some(80), Some(80), Some(80), Some(80), Some(80), Some(80), Some(80), Some(80), Some(80), 
    Some(80), Some(80), None, None, None, None, None, None, Some(79), Some(79), Some(79), Some(79), Some(79), 
    Some(79), Some(79), Some(79), Some(79), Some(79), Some(79), Some(79), Some(79), Some(79), Some(79), 
    Some(79), Some(79), Some(79), Some(79), Some(79), Some(79), None, None, None, None, Some(168), None, 
    Some(168), Some(168), Some(168), Some(168), Some(168), None, Some(168), Some(168), None, None, None, 
    None, None, None, Some(113), Some(113), Some(113), Some(113), Some(113), Some(113), Some(113), Some(113), 
    Some(113), Some(113), Some(113), Some(113), Some(113), Some(113), Some(113), Some(113), Some(113), 
    Some(113), Some(113), Some(113), Some(113), None, None, None, None, None, None, None, None, None, 
    Some(260), Some(260), Some(260), Some(260), Some(260), Some(260), Some(260), Some(260), Some(260), 
    Some(260), Some(260), Some(260), Some(260), Some(260), Some(260), Some(260), Some(260), Some(260), 
    Some(257), Some(257), None, None, Some(257), Some(257), Some(257), Some(257), Some(257), Some(185), 
    Some(257), Some(185), Some(185), Some(185), Some(185), Some(185), None, Some(185), Some(185), None, 
    None, Some(257), Some(257), Some(257), Some(257), Some(257), Some(257), Some(174), Some(174), None, 
    None, Some(174), Some(174), Some(174), Some(174), Some(174), None, Some(174), None, None, None, None, 
    None, None, None, None, None, None, Some(174), Some(174), Some(174), Some(174), Some(174), Some(174), 
    Some(156), Some(156), Some(257), Some(257), Some(156), Some(156), Some(156), Some(156), Some(156), 
    None, Some(156), None, None, None, None, None, None, None, None, None, None, Some(156), Some(156), 
    Some(156), Some(156), Some(156), Some(156), None, None, Some(174), Some(174), None, None, Some(257), 
    Some(257), None, None, None, None, None, None, Some(155), Some(155), None, None, Some(155), Some(155), 
    Some(155), Some(155), Some(155), None, Some(155), None, None, None, None, Some(156), Some(156), None, 
    None, Some(174), Some(174), Some(155), Some(155), Some(155), Some(155), Some(155), Some(155), Some(154), 
    Some(154), None, None, Some(154), Some(154), Some(154), Some(154), Some(154), None, Some(154), None, 
    None, None, None, None, None, None, None, Some(156), Some(156), Some(154), Some(154), Some(154), 
    Some(154), Some(154), Some(154), None, None, Some(155), Some(155), None, None, None, None, None, 
    None, None, None, None, None, Some(145), Some(145), None, None, Some(145), Some(145), Some(145), 
    Some(145), Some(145), None, Some(145), None, None, None, None, Some(154), Some(154), None, None, 
    Some(155), Some(155), Some(145), Some(145), Some(145), Some(145), Some(145), Some(145), None, None, 
    None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, 
    Some(154), Some(154), None, None, None, None, None, Some(74), None, None, Some(145), Some(145), Some(74), 
    None, Some(74), None, Some(74), Some(74), None, Some(74), None, None, Some(257), Some(257), Some(257), 
    Some(257), Some(257), Some(257), Some(257), Some(257), Some(257), Some(257), Some(257), Some(257), 
    Some(257), Some(257), Some(257), Some(257), Some(257), Some(257), None, Some(145), Some(145), None, 
    None, None, None, None, None, Some(174), Some(174), Some(174), Some(174), Some(174), Some(174), Some(174), 
    Some(174), Some(174), Some(174), Some(174), Some(174), Some(174), Some(174), Some(174), Some(174), 
    Some(174), Some(174), None, None, None, None, None, None, None, None, None, Some(156), Some(156), 
    Some(156), Some(156), Some(156), Some(156), Some(156), Some(156), Some(156), Some(156), Some(156), 
    Some(156), Some(156), Some(156), Some(156), Some(156), Some(156), Some(156), None, None, None, None, 
    None, Some(66), Some(74), None, None, None, None, None, None, None, None, None, None, None, None, 
    None, None, None, None, Some(155), Some(155), Some(155), Some(155), Some(155), Some(155), Some(155), 
    Some(155), Some(155), Some(155), Some(155), Some(155), Some(155), Some(155), Some(155), Some(155), 
    Some(155), Some(155), None, None, None, None, None, None, None, None, None, Some(154), Some(154), 
    Some(154), Some(154), Some(154), Some(154), Some(154), Some(154), Some(154), Some(154), Some(154), 
    Some(154), Some(154), Some(154), Some(154), Some(154), Some(154), Some(154), None, Some(47), Some(47), 
    Some(47), None, Some(47), None, None, None, None, None, None, None, None, None, None, None, None, 
    None, Some(47), Some(47), None, None, Some(145), Some(145), Some(145), Some(145), Some(145), Some(145), 
    Some(145), Some(145), Some(145), Some(145), Some(145), Some(145), Some(145), Some(145), Some(145), 
    Some(145), Some(145), Some(145), Some(91), Some(91), None, None, Some(91), Some(91), Some(91), Some(91), 
    Some(91), None, Some(91), Some(47), None, None, None, None, None, None, None, None, None, Some(91), 
    Some(91), Some(91), Some(91), Some(91), Some(91), None, None, Some(74), Some(74), Some(74), Some(74), 
    Some(74), Some(74), Some(74), None, Some(74), Some(74), None, Some(36), Some(36), Some(36), Some(47), 
    Some(36), None, None, None, None, None, None, None, None, None, None, None, Some(91), Some(91), Some(36), 
    Some(36), None, None, Some(74), Some(74), Some(74), Some(74), Some(74), Some(74), Some(74), Some(74), 
    Some(74), Some(74), Some(74), Some(74), Some(74), Some(74), None, None, None, None, Some(229), None, 
    None, Some(229), None, None, Some(229), Some(91), Some(91), Some(203), Some(203), Some(36), None, 
    Some(203), None, None, None, None, None, None, Some(229), Some(229), Some(229), None, Some(229), 
    Some(229), None, Some(203), Some(203), None, Some(203), Some(201), Some(201), Some(66), Some(66), 
    Some(201), None, None, None, None, None, None, None, Some(36), None, None, None, None, None, Some(201), 
    Some(201), None, Some(201), None, None, Some(229), Some(229), None, None, None, Some(203), Some(66), 
    Some(66), Some(66), Some(66), Some(66), Some(66), Some(66), Some(66), Some(66), Some(66), Some(66), 
    Some(66), Some(66), Some(66), Some(66), Some(66), Some(66), Some(66), Some(66), None, None, Some(201), 
    None, None, None, Some(229), Some(229), None, None, None, None, Some(203), None, None, None, Some(123), 
    None, Some(47), Some(47), None, Some(123), None, Some(123), None, Some(123), Some(123), None, Some(123), 
    None, None, None, None, None, Some(201), None, None, None, None, None, None, None, Some(123), None, 
    None, None, Some(47), Some(47), Some(47), Some(47), Some(47), Some(47), Some(47), Some(47), Some(47), 
    Some(47), Some(47), Some(47), Some(47), Some(47), Some(47), Some(47), Some(47), Some(47), Some(47), 
    None, None, None, None, None, None, Some(91), Some(91), Some(91), Some(91), Some(91), Some(91), Some(91), 
    Some(91), Some(91), Some(91), Some(91), Some(91), Some(91), Some(91), Some(91), Some(91), Some(91), 
    Some(91), None, None, None, None, None, None, None, None, None, Some(36), Some(36), None, None, None, 
    None, None, None, None, None, None, Some(123), None, None, None, None, None, None, None, None, None, 
    None, None, None, None, None, None, None, Some(36), Some(36), Some(36), Some(36), Some(36), Some(36), 
    Some(36), Some(36), Some(36), Some(36), Some(36), Some(36), Some(36), Some(36), Some(36), Some(36), 
    Some(36), Some(36), Some(36), Some(199), Some(199), None, Some(203), Some(199), Some(229), Some(229), 
    Some(229), Some(229), Some(229), Some(229), Some(229), Some(229), None, None, None, None, None, Some(199), 
    Some(199), None, Some(199), None, None, None, Some(201), None, None, None, None, Some(203), Some(203), 
    Some(203), Some(203), Some(203), Some(203), Some(203), Some(203), Some(203), Some(203), Some(203), 
    Some(203), Some(203), Some(203), Some(203), Some(203), Some(203), Some(203), Some(203), None, None, 
    Some(199), Some(201), Some(201), Some(201), Some(201), Some(201), Some(201), Some(201), Some(201), 
    Some(201), Some(201), Some(201), Some(201), Some(201), Some(201), Some(201), Some(201), Some(201), 
    Some(201), Some(201), Some(178), Some(178), Some(178), None, Some(178), None, None, None, None, None, 
    None, None, Some(199), None, None, None, None, None, Some(178), Some(178), Some(148), Some(148), 
    Some(148), None, Some(148), Some(123), None, Some(123), Some(123), Some(123), Some(123), Some(123), 
    None, Some(123), Some(123), None, None, None, Some(148), Some(148), None, None, None, None, None, 
    None, Some(136), Some(136), None, None, Some(136), Some(178), None, None, Some(232), None, None, 
    Some(232), None, None, Some(232), None, None, None, Some(136), Some(136), None, Some(136), Some(54), 
    Some(54), None, Some(148), Some(54), None, Some(232), Some(232), None, None, None, Some(232), None, 
    None, None, None, None, None, Some(54), Some(54), None, Some(54), None, None, None, None, Some(29), 
    Some(29), Some(29), Some(136), Some(29), None, None, None, None, None, None, None, None, None, None, 
    Some(232), Some(232), None, Some(29), Some(29), Some(53), Some(53), None, None, Some(53), Some(54), 
    None, None, None, None, None, None, None, None, None, Some(136), None, None, Some(53), Some(53), 
    None, Some(53), None, None, None, None, Some(232), Some(232), None, None, None, Some(29), None, None, 
    Some(199), Some(233), None, Some(54), Some(233), None, None, Some(233), None, None, None, None, None, 
    None, None, None, None, Some(53), None, None, None, Some(233), Some(233), None, None, None, Some(233), 
    Some(199), Some(199), Some(199), Some(199), Some(199), Some(199), Some(199), Some(199), Some(199), 
    Some(199), Some(199), Some(199), Some(199), Some(199), Some(199), Some(199), Some(199), Some(199), 
    Some(199), Some(27), Some(27), Some(27), Some(53), Some(27), None, None, None, None, None, Some(233), 
    Some(233), None, None, None, None, None, None, Some(27), Some(27), None, None, None, None, Some(178), 
    Some(178), None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, 
    Some(233), Some(233), None, Some(148), Some(148), None, None, None, None, None, Some(27), Some(178), 
    Some(178), Some(178), Some(178), Some(178), Some(178), Some(178), Some(178), Some(178), Some(178), 
    Some(178), Some(178), Some(178), Some(178), Some(178), Some(178), Some(178), Some(178), Some(178), 
    Some(136), Some(148), Some(148), Some(148), Some(148), Some(148), Some(148), Some(148), Some(148), 
    Some(148), Some(148), Some(148), Some(148), Some(148), Some(148), Some(148), Some(148), Some(148), 
    Some(148), Some(148), None, None, Some(54), Some(232), Some(232), Some(232), Some(232), Some(136), 
    Some(136), Some(136), Some(136), Some(136), Some(136), Some(136), Some(136), Some(136), Some(136), 
    Some(136), Some(136), Some(136), Some(136), Some(136), Some(136), Some(136), Some(136), Some(136), 
    None, Some(29), Some(29), Some(54), Some(54), Some(54), Some(54), Some(54), Some(54), Some(54), Some(54), 
    Some(54), Some(54), Some(54), Some(54), Some(54), Some(54), Some(54), Some(54), Some(54), Some(54), 
    Some(54), Some(53), None, None, None, None, None, None, Some(29), Some(29), Some(29), Some(29), Some(29), 
    Some(29), Some(29), Some(29), Some(29), Some(29), Some(29), Some(29), Some(29), Some(29), Some(29), 
    Some(29), Some(29), Some(29), Some(29), None, Some(53), Some(53), Some(53), Some(53), Some(53), Some(53), 
    Some(53), Some(53), Some(53), Some(53), Some(53), Some(53), Some(53), Some(53), Some(53), Some(53), 
    Some(53), Some(53), Some(53), Some(18), Some(18), Some(18), None, Some(18), None, Some(233), Some(233), 
    None, None, None, None, None, None, None, None, None, None, Some(18), Some(18), None, None, None, 
    None, None, None, None, None, None, None, None, None, None, Some(27), Some(27), None, None, None, 
    None, None, None, None, None, None, None, None, None, None, None, None, None, Some(18), None, None, 
    None, None, None, None, None, None, None, Some(27), Some(27), Some(27), Some(27), Some(27), Some(27), 
    Some(27), Some(27), Some(27), Some(27), Some(27), Some(27), Some(27), Some(27), Some(27), Some(27), 
    Some(27), Some(27), Some(27), Some(17), Some(17), Some(17), None, Some(17), None, None, None, None, 
    None, None, None, None, None, None, None, None, None, Some(17), Some(17), Some(16), Some(16), Some(16), 
    None, Some(16), None, None, None, None, None, None, None, None, None, None, None, None, None, Some(16), 
    Some(16), None, None, None, None, None, None, None, None, None, None, None, Some(17), None, None, 
    None, None, None, None, None, None, Some(95), None, None, Some(95), None, Some(95), Some(95), Some(95), 
    Some(15), Some(15), Some(15), Some(16), Some(15), None, None, None, None, None, None, None, Some(95), 
    Some(95), Some(95), None, Some(95), Some(95), Some(15), Some(15), None, Some(14), Some(14), Some(14), 
    None, Some(14), None, None, None, None, None, None, None, None, None, None, None, None, None, Some(14), 
    Some(14), None, None, None, None, None, None, Some(95), Some(95), None, None, Some(15), None, None, 
    None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, Some(18), 
    Some(18), Some(14), Some(13), Some(13), Some(13), Some(67), Some(13), Some(95), Some(95), None, Some(67), 
    None, Some(67), None, Some(67), Some(67), None, Some(67), None, None, Some(13), Some(13), None, None, 
    None, None, None, Some(18), Some(18), Some(18), Some(18), Some(18), Some(18), Some(18), Some(18), 
    Some(18), Some(18), Some(18), Some(18), Some(18), Some(18), Some(18), Some(18), Some(18), Some(18), 
    Some(18), None, None, None, None, Some(12), Some(12), Some(12), Some(13), Some(12), None, None, None, 
    None, None, None, None, None, None, None, Some(67), None, None, Some(12), Some(12), None, None, None, 
    None, None, None, None, None, None, Some(17), Some(17), None, None, None, None, None, None, None, 
    None, None, None, None, None, None, None, None, None, None, Some(67), Some(16), Some(16), Some(12), 
    None, None, None, None, None, Some(17), Some(17), Some(17), Some(17), Some(17), Some(17), Some(17), 
    Some(17), Some(17), Some(17), Some(17), Some(17), Some(17), Some(17), Some(17), Some(17), Some(17), 
    Some(17), Some(17), None, Some(16), Some(16), Some(16), Some(16), Some(16), Some(16), Some(16), Some(16), 
    Some(16), Some(16), Some(16), Some(16), Some(16), Some(16), Some(16), Some(16), Some(16), Some(16), 
    Some(16), None, Some(15), Some(15), None, None, Some(95), Some(95), Some(95), Some(95), Some(95), 
    Some(95), Some(95), Some(95), None, None, None, None, None, None, None, None, None, Some(14), Some(14), 
    None, None, None, None, None, Some(15), Some(15), Some(15), Some(15), Some(15), Some(15), Some(15), 
    Some(15), Some(15), Some(15), Some(15), Some(15), Some(15), Some(15), Some(15), Some(15), Some(15), 
    Some(15), Some(15), None, None, Some(14), Some(14), Some(14), Some(14), Some(14), Some(14), Some(14), 
    Some(14), Some(14), Some(14), Some(14), Some(14), Some(14), Some(14), Some(14), Some(14), Some(14), 
    Some(14), Some(14), None, None, None, None, None, Some(13), Some(13), Some(11), Some(11), Some(11), 
    None, Some(11), None, None, None, Some(67), None, Some(67), Some(67), Some(67), Some(67), Some(67), 
    None, Some(67), Some(67), Some(11), Some(11), None, None, None, None, None, None, Some(13), Some(13), 
    Some(13), Some(13), Some(13), Some(13), Some(13), Some(13), Some(13), Some(13), Some(13), Some(13), 
    Some(13), Some(13), Some(13), Some(13), Some(13), Some(13), Some(13), None, Some(12), Some(12), Some(10), 
    Some(10), Some(10), Some(11), Some(10), None, None, None, None, None, None, None, None, None, None, 
    None, None, None, Some(10), Some(10), None, None, None, None, None, None, Some(12), Some(12), Some(12), 
    Some(12), Some(12), Some(12), Some(12), Some(12), Some(12), Some(12), Some(12), Some(12), Some(12), 
    Some(12), Some(12), Some(12), Some(12), Some(12), Some(12), None, None, None, Some(9), Some(9), Some(9), 
    Some(10), Some(9), None, None, None, None, None, None, None, None, None, None, None, None, None, 
    Some(9), Some(9), Some(8), Some(8), Some(8), None, Some(8), None, None, None, None, None, None, None, 
    None, None, None, None, None, None, Some(8), Some(8), None, None, None, None, None, None, None, None, 
    None, None, None, Some(9), None, None, None, None, None, Some(223), Some(223), None, None, Some(223), 
    Some(223), Some(223), Some(223), Some(223), None, Some(223), Some(2), Some(2), Some(2), Some(8), 
    Some(2), None, None, None, None, None, Some(223), Some(223), Some(223), None, Some(223), Some(223), 
    None, None, Some(2), Some(2), Some(26), Some(26), Some(26), None, Some(26), None, None, None, None, 
    None, None, None, None, None, None, None, None, None, None, Some(26), None, None, None, None, None, 
    Some(223), Some(223), None, None, None, None, Some(2), None, None, Some(11), Some(11), None, None, 
    None, None, None, None, None, None, None, None, None, None, None, None, None, Some(26), None, None, 
    None, None, Some(223), Some(223), None, None, None, None, Some(11), Some(11), Some(11), Some(11), 
    Some(11), Some(11), Some(11), Some(11), Some(11), Some(11), Some(11), Some(11), Some(11), Some(11), 
    Some(11), Some(11), Some(11), Some(11), Some(11), None, Some(10), Some(10), Some(7), Some(7), Some(7), 
    None, Some(7), None, None, None, None, None, None, None, None, None, None, None, None, None, None, 
    Some(7), None, None, None, None, None, None, Some(10), Some(10), Some(10), Some(10), Some(10), Some(10), 
    Some(10), Some(10), Some(10), Some(10), Some(10), Some(10), Some(10), Some(10), Some(10), Some(10), 
    Some(10), Some(10), Some(10), None, Some(9), Some(9), None, None, None, Some(7), None, None, None, 
    None, None, None, None, None, None, None, None, None, None, None, Some(8), Some(8), None, None, None, 
    None, None, None, Some(9), Some(9), Some(9), Some(9), Some(9), Some(9), Some(9), Some(9), Some(9), 
    Some(9), Some(9), Some(9), Some(9), Some(9), Some(9), Some(9), Some(9), Some(9), Some(9), None, Some(8), 
    Some(8), Some(8), Some(8), Some(8), Some(8), Some(8), Some(8), Some(8), Some(8), Some(8), Some(8), 
    Some(8), Some(8), Some(8), Some(8), Some(8), Some(8), Some(8), Some(270), Some(2), Some(2), Some(223), 
    Some(223), Some(223), Some(223), Some(223), Some(223), Some(223), Some(223), None, None, None, None, 
    None, None, None, None, None, None, Some(26), Some(26), None, None, None, None, None, None, Some(2), 
    Some(2), Some(2), Some(2), Some(2), Some(2), Some(2), Some(2), Some(2), Some(2), Some(2), Some(2), 
    Some(2), Some(2), Some(2), Some(2), Some(2), Some(2), Some(2), None, Some(26), Some(26), Some(26), 
    Some(26), Some(26), Some(26), Some(26), Some(26), Some(26), Some(26), Some(26), Some(26), Some(26), 
    Some(26), Some(26), Some(26), Some(26), Some(26), Some(26), Some(6), Some(6), Some(6), None, Some(6), 
    Some(222), Some(222), None, None, Some(222), Some(222), Some(222), Some(222), Some(222), None, Some(222), 
    None, None, None, Some(6), None, None, None, None, None, None, Some(222), Some(222), Some(222), None, 
    Some(222), Some(222), None, None, None, None, None, Some(7), Some(7), None, Some(167), None, None, 
    None, None, Some(167), None, Some(167), None, Some(167), Some(167), Some(6), Some(167), None, None, 
    None, None, None, None, None, None, Some(222), Some(222), None, None, Some(7), Some(7), Some(7), 
    Some(7), Some(7), Some(7), Some(7), Some(7), Some(7), Some(7), Some(7), Some(7), Some(7), Some(7), 
    Some(7), Some(7), Some(7), Some(7), Some(7), Some(5), Some(5), Some(5), None, Some(5), None, None, 
    None, Some(222), Some(222), None, None, None, None, None, None, None, None, None, Some(5), Some(4), 
    Some(4), Some(4), None, Some(4), None, None, None, None, None, None, None, None, None, None, None, 
    None, None, None, Some(4), None, None, None, None, None, None, None, None, None, Some(167), None, 
    Some(5), None, None, None, None, None, Some(221), Some(221), None, None, Some(221), Some(221), Some(221), 
    Some(221), Some(221), Some(270), Some(221), Some(3), Some(3), Some(3), Some(4), Some(3), None, None, 
    None, None, None, Some(221), Some(221), Some(221), None, Some(221), Some(221), None, None, None, 
    Some(3), Some(251), Some(251), Some(251), None, None, Some(270), Some(270), Some(270), Some(270), 
    Some(270), Some(270), Some(270), Some(270), Some(270), Some(270), Some(270), Some(270), Some(270), 
    Some(270), Some(270), Some(270), Some(270), Some(270), Some(270), None, Some(221), Some(221), None, 
    None, None, None, Some(3), None, None, None, None, None, None, None, None, None, None, None, None, 
    None, None, Some(6), Some(6), None, None, None, Some(251), None, None, None, None, Some(221), Some(221), 
    None, None, None, None, None, None, Some(222), Some(222), Some(222), Some(222), Some(222), Some(222), 
    Some(222), Some(222), None, None, Some(6), Some(6), Some(6), Some(6), Some(6), Some(6), Some(6), 
    Some(6), Some(6), Some(6), Some(6), Some(6), Some(6), Some(6), Some(6), Some(6), Some(6), Some(6), 
    Some(6), Some(167), None, Some(167), Some(167), Some(167), Some(167), Some(167), None, Some(167), 
    Some(167), None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, 
    None, None, None, None, None, None, None, None, None, None, None, None, Some(5), Some(5), None, None, 
    None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, Some(4), 
    Some(4), None, None, None, None, None, None, Some(5), Some(5), Some(5), Some(5), Some(5), Some(5), 
    Some(5), Some(5), Some(5), Some(5), Some(5), Some(5), Some(5), Some(5), Some(5), Some(5), Some(5), 
    Some(5), Some(5), None, Some(4), Some(4), Some(4), Some(4), Some(4), Some(4), Some(4), Some(4), Some(4), 
    Some(4), Some(4), Some(4), Some(4), Some(4), Some(4), Some(4), Some(4), Some(4), Some(4), None, Some(3), 
    Some(3), Some(221), Some(221), Some(221), Some(221), Some(221), Some(221), Some(221), Some(221), 
    None, None, None, None, None, None, None, None, None, None, Some(251), Some(251), None, None, None, 
    Some(256), Some(256), Some(256), Some(3), Some(3), Some(3), Some(3), Some(3), Some(3), Some(3), Some(3), 
    Some(3), Some(3), Some(3), Some(3), Some(3), Some(3), Some(3), Some(3), Some(3), Some(3), Some(3), 
    None, Some(251), Some(251), Some(251), Some(251), Some(251), Some(251), Some(251), Some(251), Some(251), 
    Some(251), Some(251), Some(251), Some(251), Some(251), Some(251), Some(251), Some(251), Some(251), 
    Some(251), Some(220), Some(220), None, None, Some(220), Some(220), Some(220), Some(220), Some(220), 
    Some(256), Some(220), None, None, None, None, None, None, None, None, None, None, Some(220), Some(220), 
    Some(220), None, Some(220), Some(220), Some(219), Some(219), None, None, Some(219), Some(219), Some(219), 
    Some(219), Some(219), None, Some(219), None, None, None, None, None, None, None, None, None, None, 
    Some(219), Some(219), Some(219), Some(106), Some(219), Some(219), Some(94), Some(94), Some(220), 
    Some(220), Some(94), Some(94), Some(94), Some(94), Some(94), None, Some(94), None, None, None, None, 
    None, None, None, None, None, None, Some(94), Some(94), Some(94), None, Some(94), Some(94), None, 
    None, Some(219), Some(219), None, None, Some(220), Some(220), None, None, Some(106), None, Some(106), 
    None, Some(93), Some(93), None, Some(62), Some(93), Some(93), Some(93), Some(93), Some(93), None, 
    Some(93), None, None, None, None, Some(94), Some(94), None, None, Some(219), Some(219), Some(93), 
    Some(93), Some(93), Some(38), Some(93), Some(93), None, None, None, None, None, None, None, None, 
    None, None, None, None, None, None, None, None, Some(62), None, Some(62), Some(94), Some(94), None, 
    None, Some(24), None, None, None, None, None, Some(93), Some(93), None, None, None, None, None, None, 
    Some(38), None, Some(38), None, None, None, None, None, None, None, None, None, None, None, None, 
    None, None, None, Some(256), None, None, None, None, Some(93), Some(93), None, Some(24), None, Some(24), 
    None, None, None, None, None, None, None, None, None, None, None, Some(23), None, None, None, None, 
    Some(256), Some(256), Some(256), Some(256), Some(256), Some(256), Some(256), Some(256), Some(256), 
    Some(256), Some(256), Some(256), Some(256), Some(256), Some(256), Some(256), Some(256), Some(256), 
    Some(256), None, None, None, None, None, None, None, None, Some(220), Some(220), Some(220), Some(220), 
    Some(220), Some(220), Some(220), Some(220), Some(23), None, Some(23), None, None, None, None, None, 
    None, None, None, None, None, None, Some(22), None, None, None, None, Some(219), Some(219), Some(219), 
    Some(219), Some(219), Some(219), Some(219), Some(219), None, None, None, None, None, None, None, 
    None, None, None, None, None, None, None, None, None, None, None, None, Some(94), Some(94), Some(94), 
    Some(94), Some(94), Some(94), Some(94), Some(94), Some(22), None, Some(22), None, None, None, None, 
    None, None, None, None, None, None, None, None, None, Some(106), Some(106), None, Some(0), None, 
    Some(0), None, None, None, None, None, None, None, None, None, None, None, Some(93), Some(93), Some(93), 
    Some(93), Some(93), Some(93), Some(93), Some(93), None, None, None, Some(106), Some(106), Some(106), 
    Some(106), Some(106), Some(106), Some(106), Some(106), Some(106), Some(106), Some(106), Some(106), 
    Some(106), Some(106), Some(106), Some(106), Some(106), Some(106), Some(106), Some(62), Some(62), 
    None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, 
    None, None, Some(38), Some(38), None, None, None, None, None, Some(62), Some(62), Some(62), Some(62), 
    Some(62), Some(62), Some(62), Some(62), Some(62), Some(62), Some(62), Some(62), Some(62), Some(62), 
    Some(62), Some(62), Some(62), Some(62), Some(62), Some(24), Some(24), Some(38), Some(38), Some(38), 
    Some(38), Some(38), Some(38), Some(38), Some(38), Some(38), Some(38), Some(38), Some(38), Some(38), 
    Some(38), Some(38), Some(38), Some(38), Some(38), Some(38), None, None, None, None, None, None, None, 
    Some(24), Some(24), Some(24), Some(24), Some(24), Some(24), Some(24), Some(24), Some(24), Some(24), 
    Some(24), Some(24), Some(24), Some(24), Some(24), Some(24), Some(24), Some(24), Some(24), None, None, 
    None, Some(107), Some(107), Some(107), None, Some(23), Some(23), None, None, None, None, None, None, 
    None, None, None, None, None, None, Some(107), None, None, None, None, None, None, None, None, None, 
    None, None, None, None, Some(23), Some(23), Some(23), Some(23), Some(23), Some(23), Some(23), Some(23), 
    Some(23), Some(23), Some(23), Some(23), Some(23), Some(23), Some(23), Some(23), Some(23), Some(23), 
    Some(23), Some(107), None, None, None, None, None, None, Some(22), Some(22), None, None, None, None, 
    None, None, None, None, None, None, None, None, None, None, None, None, None, Some(0), Some(0), None, 
    None, None, None, None, None, None, Some(22), Some(22), Some(22), Some(22), Some(22), Some(22), Some(22), 
    Some(22), Some(22), Some(22), Some(22), Some(22), Some(22), Some(22), Some(22), Some(22), Some(22), 
    Some(22), Some(22), Some(0), Some(0), Some(0), Some(0), Some(0), Some(0), Some(0), Some(0), Some(0), 
    Some(0), Some(0), Some(0), Some(0), Some(0), Some(0), Some(0), Some(0), Some(0), Some(0), Some(228), 
    None, None, Some(228), None, None, Some(228), None, None, None, Some(227), None, None, Some(227), 
    None, None, Some(227), None, None, None, Some(228), Some(228), Some(228), Some(226), Some(228), Some(228), 
    Some(226), None, None, Some(226), Some(227), Some(227), Some(227), None, Some(227), Some(227), None, 
    None, None, None, None, None, None, Some(226), Some(226), Some(226), None, Some(226), Some(226), 
    None, None, None, None, None, None, Some(228), Some(228), None, None, None, None, None, None, None, 
    None, Some(227), Some(227), None, Some(96), None, None, Some(96), None, None, Some(96), None, None, 
    None, Some(226), Some(226), None, None, None, None, None, None, Some(228), Some(228), Some(96), Some(96), 
    Some(96), None, Some(96), Some(96), Some(107), Some(107), Some(227), Some(227), None, None, None, 
    None, None, None, None, None, Some(166), None, None, Some(226), Some(226), Some(166), None, Some(166), 
    None, Some(166), Some(166), None, Some(166), None, None, None, None, Some(96), Some(96), None, None, 
    Some(107), Some(107), Some(107), Some(107), Some(107), Some(107), Some(107), Some(107), Some(107), 
    Some(107), Some(107), Some(107), Some(107), Some(107), Some(231), None, None, Some(231), None, None, 
    Some(231), None, None, None, None, None, None, Some(96), Some(96), None, None, None, None, None, 
    Some(231), Some(231), Some(231), Some(230), Some(231), Some(231), Some(230), None, None, Some(230), 
    None, None, None, None, None, None, None, None, None, None, None, None, None, Some(230), Some(230), 
    Some(230), None, Some(230), Some(230), None, None, None, None, None, Some(165), Some(231), Some(231), 
    None, Some(166), Some(165), None, Some(165), None, Some(165), Some(165), None, Some(165), None, None, 
    None, None, None, None, None, None, None, None, None, Some(230), Some(230), None, None, None, None, 
    None, None, Some(231), Some(231), None, Some(228), Some(228), Some(228), Some(228), Some(228), Some(228), 
    Some(228), Some(228), None, None, Some(227), Some(227), Some(227), Some(227), Some(227), Some(227), 
    Some(227), Some(227), None, None, Some(230), Some(230), None, Some(226), Some(226), Some(226), Some(226), 
    Some(226), Some(226), Some(226), Some(226), Some(97), None, None, Some(97), None, None, Some(97), 
    None, None, None, None, None, None, None, None, None, None, None, None, None, Some(97), Some(97), 
    Some(97), None, Some(97), Some(97), None, Some(165), None, None, None, None, None, None, None, None, 
    None, Some(96), Some(96), Some(96), Some(96), Some(96), Some(96), Some(96), Some(96), None, None, 
    None, None, None, None, None, Some(164), None, None, Some(97), Some(97), Some(164), None, Some(164), 
    None, Some(164), Some(164), None, Some(164), None, None, None, None, None, Some(166), None, Some(166), 
    Some(166), Some(166), Some(166), Some(166), None, Some(166), Some(166), None, None, None, Some(163), 
    None, None, Some(97), Some(97), Some(163), None, Some(163), None, Some(163), Some(163), Some(92), 
    Some(163), None, None, None, Some(92), None, Some(92), None, Some(92), Some(92), None, Some(92), 
    None, None, None, None, None, Some(231), Some(231), Some(231), Some(231), Some(231), Some(231), Some(85), 
    None, None, None, None, Some(85), None, Some(85), None, Some(85), Some(85), None, Some(85), None, 
    None, None, None, Some(230), Some(230), Some(230), Some(230), Some(230), Some(230), None, None, None, 
    None, Some(164), Some(84), None, None, None, None, Some(84), None, Some(84), None, Some(84), Some(84), 
    None, Some(84), Some(165), None, Some(165), Some(165), Some(165), Some(165), Some(165), None, Some(165), 
    Some(165), None, Some(78), None, None, None, None, Some(78), Some(163), Some(78), None, Some(78), 
    Some(78), Some(77), Some(78), None, None, None, Some(77), Some(92), Some(77), None, Some(77), Some(77), 
    None, Some(77), Some(76), None, None, None, None, Some(76), None, Some(76), None, Some(76), Some(76), 
    Some(75), Some(76), None, None, None, Some(75), Some(85), Some(75), None, Some(75), Some(75), Some(73), 
    Some(75), None, None, None, Some(73), None, Some(73), None, Some(73), Some(73), None, Some(73), None, 
    None, None, Some(97), Some(97), Some(97), Some(97), Some(97), Some(97), Some(72), Some(84), None, 
    None, None, Some(72), None, Some(72), Some(98), Some(72), Some(72), Some(98), Some(72), None, Some(98), 
    None, None, None, None, None, None, None, None, None, None, Some(78), None, None, Some(98), Some(98), 
    None, None, None, Some(98), None, None, Some(77), None, None, Some(164), None, Some(164), Some(164), 
    Some(164), Some(164), Some(164), None, Some(164), Some(164), Some(76), None, None, None, None, None, 
    None, None, None, None, None, Some(75), None, None, Some(98), Some(98), None, None, None, None, None, 
    Some(163), Some(73), Some(163), Some(163), Some(163), Some(163), Some(163), None, Some(163), Some(163), 
    None, Some(92), None, Some(92), Some(92), Some(92), Some(92), Some(92), None, Some(92), Some(92), 
    None, None, Some(72), Some(98), Some(98), None, None, None, None, None, None, None, None, None, Some(85), 
    None, Some(85), Some(85), Some(85), Some(85), Some(85), None, Some(85), Some(85), None, None, None, 
    None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, Some(84), 
    None, Some(84), Some(84), Some(84), Some(84), Some(84), None, Some(84), Some(84), None, None, None, 
    None, None, None, None, None, None, None, None, None, None, None, Some(78), None, Some(78), Some(78), 
    Some(78), Some(78), Some(78), None, Some(78), Some(78), None, Some(77), None, Some(77), Some(77), 
    Some(77), Some(77), Some(77), None, Some(77), Some(77), None, None, None, Some(76), None, Some(76), 
    Some(76), Some(76), Some(76), Some(76), None, Some(76), Some(76), None, Some(75), None, Some(75), 
    Some(75), Some(75), Some(75), Some(75), None, Some(75), Some(75), None, Some(73), None, Some(73), 
    Some(73), Some(73), Some(73), Some(73), None, Some(73), Some(73), None, None, Some(99), None, None, 
    Some(99), None, None, Some(99), None, None, None, Some(72), None, Some(72), Some(72), Some(72), Some(72), 
    Some(72), None, Some(72), Some(72), Some(99), Some(99), None, None, None, Some(99), None, Some(98), 
    Some(98), Some(98), Some(98), None, None, None, None, None, None, None, None, None, None, None, None, 
    None, None, None, None, None, None, None, None, None, None, None, None, Some(99), Some(99), None, 
    None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, 
    None, None, None, None, None, None, None, None, None, None, None, Some(99), Some(99), None, None, 
    None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, 
    None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, 
    None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, 
    None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, 
    None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, 
    None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, 
    None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, 
    None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, 
    None, None, None, None, None, None, None, None, None, None, Some(99), Some(99), 
];

static ACTION_ROW_ID: [usize; 348] = [
    0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 0, 22, 23, 24, 25, 
    26, 26, 26, 27, 28, 29, 30, 31, 32, 33, 34, 35, 36, 37, 38, 39, 40, 41, 42, 43, 44, 45, 46, 47, 48, 
    49, 50, 51, 25, 52, 53, 54, 55, 56, 57, 58, 52, 59, 49, 60, 61, 62, 63, 64, 65, 66, 67, 68, 69, 70, 
    71, 72, 73, 74, 75, 76, 77, 59, 78, 79, 80, 81, 82, 83, 84, 85, 85, 86, 87, 88, 89, 90, 91, 92, 93, 
    94, 95, 96, 97, 98, 99, 100, 101, 102, 103, 104, 105, 106, 107, 107, 108, 109, 110, 111, 112, 113, 
    92, 114, 115, 116, 117, 118, 119, 120, 121, 122, 123, 124, 125, 126, 127, 128, 129, 130, 131, 132, 
    133, 134, 135, 136, 137, 138, 139, 140, 141, 142, 143, 144, 145, 146, 147, 92, 57, 148, 149, 150, 
    151, 152, 153, 74, 154, 92, 155, 156, 157, 158, 159, 92, 160, 161, 162, 163, 164, 165, 166, 167, 
    168, 169, 170, 171, 172, 173, 92, 174, 92, 92, 92, 92, 92, 92, 92, 92, 92, 92, 92, 92, 92, 92, 92, 
    92, 92, 92, 92, 175, 176, 177, 178, 179, 92, 180, 181, 182, 118, 183, 118, 92, 92, 92, 184, 185, 
    186, 187, 188, 189, 190, 191, 192, 92, 193, 194, 195, 196, 197, 198, 199, 200, 201, 202, 203, 204, 
    205, 206, 207, 208, 92, 209, 210, 211, 212, 213, 214, 215, 216, 217, 218, 219, 220, 221, 222, 223, 
    224, 225, 226, 227, 228, 229, 230, 231, 232, 233, 234, 235, 236, 237, 238, 110, 239, 92, 240, 118, 
    241, 242, 243, 244, 245, 246, 247, 248, 249, 250, 251, 67, 252, 253, 254, 255, 256, 257, 258, 259, 
    260, 261, 92, 262, 92, 263, 264, 265, 118, 118, 118, 92, 185, 266, 267, 268, 269, 270, 67, 271, 272, 
    273, 274, 275, 276, 277, 278, 279, 280, 281, 282, 118, 283, 284, 285, 286, 287, 288, 289, 118, 290, 
    
];


/// goto matrix -> base next check
static GOTO_BASE: [Option<usize>; 348] = [
    Some(1232), Some(98), None, Some(99), Some(1275), Some(1327), Some(1502), Some(1133), Some(578), 
    Some(94), Some(1428), Some(78), Some(63), Some(13), Some(13), Some(1495), Some(13), Some(387), Some(5), 
    Some(639), Some(864), Some(12), Some(1182), Some(0), Some(1309), Some(832), Some(463), Some(544), 
    Some(1197), Some(856), Some(639), Some(5), Some(1527), Some(1556), Some(1544), Some(19), Some(1563), 
    Some(1020), Some(374), Some(806), Some(69), Some(1255), Some(996), Some(10), Some(483), Some(397), 
    Some(783), Some(511), Some(760), Some(975), Some(1494), Some(1485), Some(1476), Some(1437), Some(1424), 
    Some(1404), Some(1393), Some(1366), Some(1354), Some(1342), Some(1330), Some(1317), Some(1304), Some(1251), 
    Some(1208), Some(1163), Some(1114), Some(737), Some(1064), Some(951), Some(333), Some(300), Some(714), 
    Some(691), Some(668), Some(488), Some(930), Some(47), Some(1459), Some(12), Some(1467), Some(647), 
    Some(486), Some(906), Some(267), Some(1126), Some(615), Some(1175), Some(885), Some(1044), Some(234), 
    Some(201), Some(168), Some(589), Some(434), Some(1316), Some(565), Some(135), Some(34), Some(102), 
    None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, 
    None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, 
    None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, 
    None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, 
    None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, 
    None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, 
    None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, 
    None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, 
    None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, 
    None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, 
    None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, 
    None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, 
    None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, 
    None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, 
    None, None, None, None, None, None, None, None, None, None, 
];

static GOTO_NEXT: [Option<usize>; 1592] = [
    None, None, None, None, None, Some(134), None, None, None, Some(55), None, Some(27), Some(28), Some(29), 
    Some(30), Some(31), Some(65), Some(53), Some(54), Some(42), Some(43), Some(44), Some(55), Some(32), 
    Some(27), Some(28), Some(29), Some(30), Some(31), Some(75), Some(76), Some(115), Some(116), Some(30), 
    Some(31), Some(247), Some(32), Some(214), Some(119), Some(70), Some(34), Some(35), Some(32), Some(135), 
    Some(136), Some(137), Some(138), Some(139), Some(140), Some(141), Some(142), Some(299), Some(143), 
    Some(95), Some(96), Some(97), Some(98), Some(114), Some(68), Some(99), Some(100), Some(101), Some(102), 
    Some(103), Some(104), Some(105), Some(106), Some(107), Some(108), Some(109), Some(110), Some(111), 
    Some(112), Some(144), Some(134), Some(145), Some(59), Some(58), Some(55), Some(189), Some(27), Some(28), 
    Some(29), Some(30), Some(31), Some(345), Some(299), Some(95), Some(96), Some(97), Some(98), Some(60), 
    Some(32), Some(99), Some(100), Some(101), Some(102), Some(103), Some(104), Some(105), Some(106), 
    Some(107), Some(108), Some(109), Some(110), Some(111), Some(112), Some(144), Some(58), Some(293), 
    Some(51), Some(36), Some(135), Some(136), Some(137), Some(40), Some(233), Some(140), Some(141), Some(142), 
    None, Some(143), Some(95), Some(96), Some(97), Some(98), Some(37), Some(38), Some(99), Some(100), 
    Some(101), Some(102), Some(103), Some(104), Some(105), Some(106), Some(107), Some(108), Some(109), 
    Some(110), Some(111), Some(112), Some(144), None, Some(145), Some(347), Some(136), Some(137), None, 
    None, Some(140), Some(141), Some(142), None, Some(143), Some(95), Some(96), Some(97), Some(98), None, 
    None, Some(99), Some(100), Some(101), Some(102), Some(103), Some(104), Some(105), Some(106), Some(107), 
    Some(108), Some(109), Some(110), Some(111), Some(112), Some(144), None, Some(145), Some(343), Some(136), 
    Some(137), None, None, Some(140), Some(141), Some(142), None, Some(143), Some(95), Some(96), Some(97), 
    Some(98), None, None, Some(99), Some(100), Some(101), Some(102), Some(103), Some(104), Some(105), 
    Some(106), Some(107), Some(108), Some(109), Some(110), Some(111), Some(112), Some(144), None, Some(145), 
    Some(330), Some(136), Some(137), None, None, Some(140), Some(141), Some(142), None, Some(143), Some(95), 
    Some(96), Some(97), Some(98), None, None, Some(99), Some(100), Some(101), Some(102), Some(103), Some(104), 
    Some(105), Some(106), Some(107), Some(108), Some(109), Some(110), Some(111), Some(112), Some(144), 
    None, Some(145), Some(329), Some(136), Some(137), None, None, Some(140), Some(141), Some(142), None, 
    Some(143), Some(95), Some(96), Some(97), Some(98), None, None, Some(99), Some(100), Some(101), Some(102), 
    Some(103), Some(104), Some(105), Some(106), Some(107), Some(108), Some(109), Some(110), Some(111), 
    Some(112), Some(144), None, Some(145), Some(328), Some(136), Some(137), None, None, Some(140), Some(141), 
    Some(142), None, Some(143), Some(95), Some(96), Some(97), Some(98), None, None, Some(99), Some(100), 
    Some(101), Some(102), Some(103), Some(104), Some(105), Some(106), Some(107), Some(108), Some(109), 
    Some(110), Some(111), Some(112), Some(144), None, Some(145), Some(314), Some(136), Some(137), None, 
    None, Some(140), Some(141), Some(142), None, Some(143), Some(95), Some(96), Some(97), Some(98), None, 
    None, Some(99), Some(100), Some(101), Some(102), Some(103), Some(104), Some(105), Some(106), Some(107), 
    Some(108), Some(109), Some(110), Some(111), Some(112), Some(144), None, Some(145), Some(287), Some(136), 
    Some(137), None, None, Some(140), Some(141), Some(142), None, Some(143), Some(95), Some(96), Some(97), 
    Some(98), None, None, Some(99), Some(100), Some(101), Some(102), Some(103), Some(104), Some(105), 
    Some(106), Some(107), Some(108), Some(109), Some(110), Some(111), Some(112), Some(144), None, Some(145), 
    Some(285), Some(136), Some(137), None, None, Some(140), Some(141), Some(142), None, Some(143), Some(95), 
    Some(96), Some(97), Some(98), None, None, Some(99), Some(100), Some(101), Some(102), Some(103), Some(104), 
    Some(105), Some(106), Some(107), Some(108), Some(109), Some(110), Some(111), Some(112), Some(144), 
    None, Some(145), Some(115), Some(116), Some(30), Some(31), Some(71), Some(34), Some(35), Some(161), 
    Some(225), Some(136), Some(137), Some(32), None, Some(140), Some(141), Some(142), None, Some(143), 
    Some(95), Some(96), Some(97), Some(98), None, None, Some(99), Some(100), Some(101), Some(102), Some(103), 
    Some(104), Some(105), Some(106), Some(107), Some(108), Some(109), Some(110), Some(111), Some(112), 
    Some(144), None, Some(145), Some(95), Some(96), Some(97), Some(98), None, None, Some(99), Some(100), 
    Some(101), Some(102), Some(103), Some(104), Some(105), Some(106), Some(107), Some(108), Some(109), 
    Some(110), Some(111), Some(112), Some(144), None, Some(162), None, Some(254), Some(115), Some(116), 
    Some(30), Some(31), None, None, None, Some(161), None, None, Some(332), Some(32), Some(95), Some(96), 
    Some(97), Some(98), None, None, Some(99), Some(100), Some(101), Some(102), Some(103), Some(104), 
    Some(105), Some(106), Some(107), Some(108), Some(109), Some(110), Some(111), Some(112), Some(144), 
    Some(312), Some(293), Some(248), Some(238), Some(218), Some(34), Some(35), None, Some(95), Some(96), 
    Some(97), Some(98), Some(249), Some(240), Some(99), Some(100), Some(101), Some(102), Some(103), Some(104), 
    Some(105), Some(106), Some(107), Some(108), Some(109), Some(110), Some(111), Some(112), Some(144), 
    None, Some(162), Some(292), Some(163), Some(95), Some(96), Some(97), Some(98), None, None, Some(99), 
    Some(100), Some(101), Some(102), Some(103), Some(104), Some(105), Some(106), Some(107), Some(108), 
    Some(109), Some(110), Some(111), Some(112), Some(144), None, Some(293), Some(95), Some(96), Some(97), 
    Some(98), Some(255), Some(256), Some(99), Some(100), Some(101), Some(102), Some(103), Some(104), 
    Some(105), Some(106), Some(107), Some(108), Some(109), Some(110), Some(111), Some(112), Some(257), 
    Some(164), Some(165), Some(46), Some(49), Some(27), Some(28), Some(29), Some(30), Some(31), None, 
    None, Some(337), Some(95), Some(96), Some(97), Some(98), Some(32), None, Some(99), Some(100), Some(101), 
    Some(102), Some(103), Some(104), Some(105), Some(106), Some(107), Some(108), Some(109), Some(110), 
    Some(111), Some(112), Some(113), Some(95), Some(96), Some(97), Some(98), None, None, Some(155), Some(100), 
    Some(101), Some(102), Some(103), Some(104), Some(105), Some(106), Some(107), Some(108), Some(109), 
    Some(110), Some(111), Some(156), None, None, None, Some(157), Some(95), Some(96), Some(97), Some(98), 
    Some(323), Some(77), Some(99), Some(100), Some(101), Some(102), Some(103), Some(104), Some(105), 
    Some(106), Some(107), Some(108), Some(109), Some(110), Some(111), Some(112), Some(144), None, Some(331), 
    Some(70), Some(34), Some(35), Some(95), Some(96), Some(97), Some(98), None, None, Some(155), Some(100), 
    Some(101), Some(102), Some(103), Some(104), Some(105), Some(106), Some(107), Some(108), Some(109), 
    Some(110), Some(111), Some(156), Some(306), None, None, Some(157), Some(95), Some(96), Some(97), 
    Some(98), None, None, Some(170), Some(100), Some(95), Some(96), Some(97), Some(98), None, None, Some(99), 
    Some(100), Some(101), Some(102), Some(103), Some(104), Some(105), Some(106), Some(107), Some(108), 
    Some(109), Some(110), Some(111), Some(112), Some(113), Some(95), Some(96), Some(97), Some(98), None, 
    None, Some(99), Some(100), Some(101), Some(102), Some(103), Some(104), Some(105), Some(106), Some(107), 
    Some(108), Some(109), Some(110), Some(111), Some(112), Some(144), None, Some(290), Some(95), Some(96), 
    Some(97), Some(98), None, None, Some(99), Some(100), Some(101), Some(102), Some(103), Some(104), 
    Some(105), Some(106), Some(107), Some(108), Some(109), Some(110), Some(111), Some(112), Some(144), 
    None, Some(289), Some(95), Some(96), Some(97), Some(98), None, None, Some(99), Some(100), Some(101), 
    Some(102), Some(103), Some(104), Some(105), Some(106), Some(107), Some(108), Some(109), Some(110), 
    Some(111), Some(112), Some(144), None, Some(288), Some(95), Some(96), Some(97), Some(98), None, None, 
    Some(99), Some(100), Some(101), Some(102), Some(103), Some(104), Some(105), Some(106), Some(107), 
    Some(108), Some(109), Some(110), Some(111), Some(112), Some(144), None, Some(279), Some(95), Some(96), 
    Some(97), Some(98), None, None, Some(99), Some(100), Some(101), Some(102), Some(103), Some(104), 
    Some(105), Some(106), Some(107), Some(108), Some(109), Some(110), Some(111), Some(112), Some(144), 
    None, Some(259), Some(95), Some(96), Some(97), Some(98), None, None, Some(99), Some(100), Some(101), 
    Some(102), Some(103), Some(104), Some(105), Some(106), Some(107), Some(108), Some(109), Some(110), 
    Some(111), Some(112), Some(144), None, Some(162), Some(95), Some(96), Some(97), Some(98), Some(154), 
    None, Some(99), Some(100), Some(101), Some(102), Some(103), Some(104), Some(105), Some(106), Some(107), 
    Some(108), Some(109), Some(110), Some(111), Some(112), Some(144), None, Some(231), None, None, None, 
    Some(95), Some(96), Some(97), Some(98), None, None, Some(155), Some(100), Some(101), Some(102), Some(103), 
    Some(104), Some(105), Some(106), Some(107), Some(108), Some(109), Some(110), Some(111), Some(156), 
    Some(94), None, None, Some(157), Some(95), Some(96), Some(97), Some(98), None, None, Some(169), Some(100), 
    Some(95), Some(96), Some(97), Some(98), None, None, Some(99), Some(100), Some(101), Some(102), Some(103), 
    Some(104), Some(105), Some(106), Some(107), Some(108), Some(109), Some(110), Some(111), Some(112), 
    Some(113), Some(95), Some(96), Some(97), Some(98), None, None, Some(99), Some(100), Some(101), Some(102), 
    Some(103), Some(104), Some(105), Some(106), Some(107), Some(108), Some(109), Some(110), Some(111), 
    Some(112), Some(326), Some(95), Some(96), Some(97), Some(98), None, None, Some(155), Some(100), Some(101), 
    Some(102), Some(103), Some(104), Some(105), Some(106), Some(107), Some(108), Some(109), Some(110), 
    Some(111), Some(156), None, None, None, Some(313), Some(95), Some(96), Some(97), Some(98), None, 
    None, Some(99), Some(100), Some(101), Some(102), Some(103), Some(104), Some(105), Some(106), Some(107), 
    Some(108), Some(109), Some(110), Some(111), Some(112), Some(296), Some(95), Some(96), Some(97), Some(98), 
    None, None, Some(155), Some(100), Some(101), Some(102), Some(103), Some(104), Some(105), Some(106), 
    Some(107), Some(108), Some(109), Some(110), Some(111), Some(156), None, None, None, Some(281), Some(95), 
    Some(96), Some(97), Some(98), None, None, Some(99), Some(100), Some(101), Some(102), Some(103), Some(104), 
    Some(105), Some(106), Some(107), Some(108), Some(109), Some(110), Some(111), Some(112), Some(261), 
    Some(95), Some(96), Some(97), Some(98), None, None, Some(155), Some(100), Some(101), Some(102), Some(103), 
    Some(104), Some(105), Some(106), Some(107), Some(108), Some(109), Some(110), Some(111), Some(156), 
    None, None, None, Some(246), Some(95), Some(96), Some(97), Some(98), None, None, Some(155), Some(100), 
    Some(101), Some(102), Some(103), Some(104), Some(105), Some(106), Some(107), Some(108), Some(109), 
    Some(110), Some(111), Some(156), None, None, None, Some(220), Some(95), Some(96), Some(97), Some(98), 
    None, None, Some(155), Some(100), Some(101), Some(102), Some(103), Some(104), Some(105), Some(106), 
    Some(107), Some(108), Some(109), Some(110), Some(111), Some(327), Some(95), Some(96), Some(97), Some(98), 
    None, None, Some(155), Some(100), Some(101), Some(102), Some(103), Some(104), Some(105), Some(106), 
    Some(107), Some(108), Some(109), Some(280), Some(148), None, Some(27), Some(28), Some(29), Some(30), 
    Some(31), Some(46), Some(48), Some(27), Some(28), Some(29), Some(30), Some(31), Some(32), None, None, 
    Some(71), Some(237), Some(238), None, Some(32), None, None, None, Some(320), Some(152), Some(153), 
    None, Some(321), None, Some(322), Some(95), Some(96), Some(97), Some(98), None, None, Some(155), 
    Some(100), Some(101), Some(102), Some(103), Some(104), Some(105), Some(106), Some(107), Some(108), 
    Some(278), Some(148), None, Some(27), Some(28), Some(29), Some(30), Some(31), None, None, None, Some(115), 
    Some(116), Some(30), Some(31), Some(32), Some(117), Some(118), Some(119), Some(248), Some(238), None, 
    Some(32), None, None, None, Some(320), Some(152), Some(153), None, Some(321), None, Some(322), Some(95), 
    Some(96), Some(97), Some(98), None, None, Some(155), Some(100), Some(101), Some(102), Some(103), 
    Some(104), Some(105), Some(106), Some(107), Some(277), Some(22), Some(23), Some(24), None, None, 
    Some(25), None, None, None, Some(26), None, Some(27), Some(28), Some(29), Some(30), Some(31), None, 
    None, Some(95), Some(96), Some(97), Some(98), None, Some(32), Some(167), Some(100), Some(33), Some(34), 
    Some(35), Some(95), Some(96), Some(97), Some(98), None, None, Some(155), Some(100), Some(101), Some(102), 
    Some(103), Some(104), Some(105), Some(106), Some(276), Some(41), Some(24), None, None, Some(25), 
    Some(236), Some(237), Some(238), Some(26), None, Some(27), Some(28), Some(29), Some(30), Some(31), 
    None, Some(239), Some(240), None, None, None, None, Some(32), None, None, Some(33), Some(34), Some(35), 
    Some(95), Some(96), Some(97), Some(98), None, None, Some(155), Some(100), Some(101), Some(102), Some(103), 
    Some(104), Some(105), Some(275), Some(148), None, Some(27), Some(28), Some(29), Some(30), Some(31), 
    Some(148), None, Some(27), Some(28), Some(29), Some(30), Some(31), Some(32), Some(42), Some(43), 
    Some(44), None, None, None, Some(32), None, Some(149), Some(150), Some(151), Some(152), Some(153), 
    None, None, None, None, Some(320), Some(152), Some(153), Some(45), Some(34), Some(35), Some(336), 
    Some(95), Some(96), Some(97), Some(98), None, None, Some(155), Some(100), Some(101), Some(102), Some(103), 
    Some(104), Some(274), Some(95), Some(96), Some(97), Some(98), None, None, Some(155), Some(100), Some(101), 
    Some(102), Some(103), Some(104), Some(273), Some(95), Some(96), Some(97), Some(98), None, None, Some(155), 
    Some(100), Some(101), Some(102), Some(103), Some(272), Some(95), Some(96), Some(97), Some(98), None, 
    None, Some(155), Some(100), Some(101), Some(102), Some(103), Some(271), Some(95), Some(96), Some(97), 
    Some(98), None, None, Some(155), Some(100), Some(101), Some(102), Some(103), Some(270), Some(95), 
    Some(96), Some(97), Some(98), None, None, Some(155), Some(100), Some(101), Some(102), Some(103), 
    Some(269), Some(52), Some(53), Some(54), None, None, None, Some(55), None, Some(27), Some(28), Some(29), 
    Some(30), Some(31), None, None, Some(95), Some(96), Some(97), Some(98), None, Some(32), Some(155), 
    Some(100), Some(101), Some(102), Some(268), Some(95), Some(96), Some(97), Some(98), None, None, Some(155), 
    Some(100), Some(101), Some(102), Some(267), Some(148), None, Some(27), Some(28), Some(29), Some(30), 
    Some(31), None, None, Some(95), Some(96), Some(97), Some(98), None, Some(32), Some(155), Some(100), 
    Some(101), Some(266), None, None, None, Some(95), Some(96), Some(97), Some(98), None, Some(302), 
    Some(155), Some(100), Some(101), Some(265), Some(69), None, None, None, Some(55), None, Some(27), 
    Some(28), Some(29), Some(30), Some(31), Some(46), Some(47), Some(27), Some(28), Some(29), Some(30), 
    Some(31), Some(32), None, Some(95), Some(96), Some(97), Some(98), None, Some(32), Some(155), Some(100), 
    Some(304), Some(95), Some(96), Some(97), Some(98), None, None, Some(155), Some(100), Some(264), Some(95), 
    Some(96), Some(97), Some(98), None, None, Some(155), Some(100), Some(263), Some(95), Some(96), Some(97), 
    Some(98), None, None, Some(155), Some(100), Some(262), Some(115), Some(116), Some(30), Some(31), 
    None, None, None, Some(210), Some(212), None, None, Some(32), Some(115), Some(116), Some(30), Some(31), 
    None, None, None, Some(210), Some(211), None, None, Some(32), Some(95), Some(96), Some(97), Some(98), 
    Some(216), Some(217), Some(155), Some(100), Some(190), Some(218), Some(34), Some(35), 
];

static GOTO_CHECK: [Option<usize>; 1592] = [
    None, None, None, None, None, Some(23), None, None, None, Some(23), None, Some(23), Some(23), Some(23), 
    Some(23), Some(23), Some(13), Some(13), Some(13), Some(16), Some(16), Some(16), Some(13), Some(23), 
    Some(13), Some(13), Some(13), Some(13), Some(13), Some(18), Some(18), Some(35), Some(35), Some(35), 
    Some(35), Some(43), Some(13), Some(35), Some(35), Some(16), Some(16), Some(16), Some(35), Some(23), 
    Some(23), Some(23), Some(23), Some(23), Some(23), Some(23), Some(23), Some(79), Some(23), Some(23), 
    Some(23), Some(23), Some(23), Some(21), Some(14), Some(23), Some(23), Some(23), Some(23), Some(23), 
    Some(23), Some(23), Some(23), Some(23), Some(23), Some(23), Some(23), Some(23), Some(23), Some(23), 
    Some(40), Some(23), Some(12), Some(77), Some(40), Some(31), Some(40), Some(40), Some(40), Some(40), 
    Some(40), Some(98), Some(77), Some(98), Some(98), Some(98), Some(98), Some(12), Some(40), Some(98), 
    Some(98), Some(98), Some(98), Some(98), Some(98), Some(98), Some(98), Some(98), Some(98), Some(98), 
    Some(98), Some(98), Some(98), Some(98), Some(11), Some(98), Some(9), Some(1), Some(40), Some(40), 
    Some(40), Some(3), Some(40), Some(40), Some(40), Some(40), None, Some(40), Some(40), Some(40), Some(40), 
    Some(40), Some(1), Some(1), Some(40), Some(40), Some(40), Some(40), Some(40), Some(40), Some(40), 
    Some(40), Some(40), Some(40), Some(40), Some(40), Some(40), Some(40), Some(40), None, Some(40), Some(99), 
    Some(99), Some(99), None, None, Some(99), Some(99), Some(99), None, Some(99), Some(99), Some(99), 
    Some(99), Some(99), None, None, Some(99), Some(99), Some(99), Some(99), Some(99), Some(99), Some(99), 
    Some(99), Some(99), Some(99), Some(99), Some(99), Some(99), Some(99), Some(99), None, Some(99), Some(97), 
    Some(97), Some(97), None, None, Some(97), Some(97), Some(97), None, Some(97), Some(97), Some(97), 
    Some(97), Some(97), None, None, Some(97), Some(97), Some(97), Some(97), Some(97), Some(97), Some(97), 
    Some(97), Some(97), Some(97), Some(97), Some(97), Some(97), Some(97), Some(97), None, Some(97), Some(92), 
    Some(92), Some(92), None, None, Some(92), Some(92), Some(92), None, Some(92), Some(92), Some(92), 
    Some(92), Some(92), None, None, Some(92), Some(92), Some(92), Some(92), Some(92), Some(92), Some(92), 
    Some(92), Some(92), Some(92), Some(92), Some(92), Some(92), Some(92), Some(92), None, Some(92), Some(91), 
    Some(91), Some(91), None, None, Some(91), Some(91), Some(91), None, Some(91), Some(91), Some(91), 
    Some(91), Some(91), None, None, Some(91), Some(91), Some(91), Some(91), Some(91), Some(91), Some(91), 
    Some(91), Some(91), Some(91), Some(91), Some(91), Some(91), Some(91), Some(91), None, Some(91), Some(90), 
    Some(90), Some(90), None, None, Some(90), Some(90), Some(90), None, Some(90), Some(90), Some(90), 
    Some(90), Some(90), None, None, Some(90), Some(90), Some(90), Some(90), Some(90), Some(90), Some(90), 
    Some(90), Some(90), Some(90), Some(90), Some(90), Some(90), Some(90), Some(90), None, Some(90), Some(84), 
    Some(84), Some(84), None, None, Some(84), Some(84), Some(84), None, Some(84), Some(84), Some(84), 
    Some(84), Some(84), None, None, Some(84), Some(84), Some(84), Some(84), Some(84), Some(84), Some(84), 
    Some(84), Some(84), Some(84), Some(84), Some(84), Some(84), Some(84), Some(84), None, Some(84), Some(71), 
    Some(71), Some(71), None, None, Some(71), Some(71), Some(71), None, Some(71), Some(71), Some(71), 
    Some(71), Some(71), None, None, Some(71), Some(71), Some(71), Some(71), Some(71), Some(71), Some(71), 
    Some(71), Some(71), Some(71), Some(71), Some(71), Some(71), Some(71), Some(71), None, Some(71), Some(70), 
    Some(70), Some(70), None, None, Some(70), Some(70), Some(70), None, Some(70), Some(70), Some(70), 
    Some(70), Some(70), None, None, Some(70), Some(70), Some(70), Some(70), Some(70), Some(70), Some(70), 
    Some(70), Some(70), Some(70), Some(70), Some(70), Some(70), Some(70), Some(70), None, Some(70), Some(45), 
    Some(45), Some(45), Some(45), Some(17), Some(17), Some(17), Some(45), Some(38), Some(38), Some(38), 
    Some(45), None, Some(38), Some(38), Some(38), None, Some(38), Some(38), Some(38), Some(38), Some(38), 
    None, None, Some(38), Some(38), Some(38), Some(38), Some(38), Some(38), Some(38), Some(38), Some(38), 
    Some(38), Some(38), Some(38), Some(38), Some(38), Some(38), None, Some(38), Some(45), Some(45), Some(45), 
    Some(45), None, None, Some(45), Some(45), Some(45), Some(45), Some(45), Some(45), Some(45), Some(45), 
    Some(45), Some(45), Some(45), Some(45), Some(45), Some(45), Some(45), None, Some(45), None, Some(45), 
    Some(26), Some(26), Some(26), Some(26), None, None, None, Some(26), None, None, Some(94), Some(26), 
    Some(94), Some(94), Some(94), Some(94), None, None, Some(94), Some(94), Some(94), Some(94), Some(94), 
    Some(94), Some(94), Some(94), Some(94), Some(94), Some(94), Some(94), Some(94), Some(94), Some(94), 
    Some(82), Some(94), Some(44), Some(44), Some(82), Some(82), Some(82), None, Some(26), Some(26), Some(26), 
    Some(26), Some(44), Some(44), Some(26), Some(26), Some(26), Some(26), Some(26), Some(26), Some(26), 
    Some(26), Some(26), Some(26), Some(26), Some(26), Some(26), Some(26), Some(26), None, Some(26), Some(75), 
    Some(26), Some(75), Some(75), Some(75), Some(75), None, None, Some(75), Some(75), Some(75), Some(75), 
    Some(75), Some(75), Some(75), Some(75), Some(75), Some(75), Some(75), Some(75), Some(75), Some(75), 
    Some(75), None, Some(75), Some(47), Some(47), Some(47), Some(47), Some(47), Some(47), Some(47), Some(47), 
    Some(47), Some(47), Some(47), Some(47), Some(47), Some(47), Some(47), Some(47), Some(47), Some(47), 
    Some(47), Some(47), Some(47), Some(27), Some(27), Some(8), Some(8), Some(8), Some(8), Some(8), Some(8), 
    Some(8), None, None, Some(96), Some(27), Some(27), Some(27), Some(27), Some(8), None, Some(27), Some(27), 
    Some(27), Some(27), Some(27), Some(27), Some(27), Some(27), Some(27), Some(27), Some(27), Some(27), 
    Some(27), Some(27), Some(27), Some(96), Some(96), Some(96), Some(96), None, None, Some(96), Some(96), 
    Some(96), Some(96), Some(96), Some(96), Some(96), Some(96), Some(96), Some(96), Some(96), Some(96), 
    Some(96), Some(96), None, None, None, Some(96), Some(93), Some(93), Some(93), Some(93), Some(86), 
    Some(19), Some(93), Some(93), Some(93), Some(93), Some(93), Some(93), Some(93), Some(93), Some(93), 
    Some(93), Some(93), Some(93), Some(93), Some(93), Some(93), None, Some(93), Some(19), Some(19), Some(19), 
    Some(86), Some(86), Some(86), Some(86), None, None, Some(86), Some(86), Some(86), Some(86), Some(86), 
    Some(86), Some(86), Some(86), Some(86), Some(86), Some(86), Some(86), Some(86), Some(86), Some(81), 
    None, None, Some(86), Some(30), Some(30), Some(30), Some(30), None, None, Some(30), Some(30), Some(81), 
    Some(81), Some(81), Some(81), None, None, Some(81), Some(81), Some(81), Some(81), Some(81), Some(81), 
    Some(81), Some(81), Some(81), Some(81), Some(81), Some(81), Some(81), Some(81), Some(81), Some(74), 
    Some(74), Some(74), Some(74), None, None, Some(74), Some(74), Some(74), Some(74), Some(74), Some(74), 
    Some(74), Some(74), Some(74), Some(74), Some(74), Some(74), Some(74), Some(74), Some(74), None, Some(74), 
    Some(73), Some(73), Some(73), Some(73), None, None, Some(73), Some(73), Some(73), Some(73), Some(73), 
    Some(73), Some(73), Some(73), Some(73), Some(73), Some(73), Some(73), Some(73), Some(73), Some(73), 
    None, Some(73), Some(72), Some(72), Some(72), Some(72), None, None, Some(72), Some(72), Some(72), 
    Some(72), Some(72), Some(72), Some(72), Some(72), Some(72), Some(72), Some(72), Some(72), Some(72), 
    Some(72), Some(72), None, Some(72), Some(67), Some(67), Some(67), Some(67), None, None, Some(67), 
    Some(67), Some(67), Some(67), Some(67), Some(67), Some(67), Some(67), Some(67), Some(67), Some(67), 
    Some(67), Some(67), Some(67), Some(67), None, Some(67), Some(48), Some(48), Some(48), Some(48), None, 
    None, Some(48), Some(48), Some(48), Some(48), Some(48), Some(48), Some(48), Some(48), Some(48), Some(48), 
    Some(48), Some(48), Some(48), Some(48), Some(48), None, Some(48), Some(46), Some(46), Some(46), Some(46), 
    None, None, Some(46), Some(46), Some(46), Some(46), Some(46), Some(46), Some(46), Some(46), Some(46), 
    Some(46), Some(46), Some(46), Some(46), Some(46), Some(46), None, Some(46), Some(39), Some(39), Some(39), 
    Some(39), Some(25), None, Some(39), Some(39), Some(39), Some(39), Some(39), Some(39), Some(39), Some(39), 
    Some(39), Some(39), Some(39), Some(39), Some(39), Some(39), Some(39), None, Some(39), None, None, 
    None, Some(25), Some(25), Some(25), Some(25), None, None, Some(25), Some(25), Some(25), Some(25), 
    Some(25), Some(25), Some(25), Some(25), Some(25), Some(25), Some(25), Some(25), Some(25), Some(25), 
    Some(20), None, None, Some(25), Some(29), Some(29), Some(29), Some(29), None, None, Some(29), Some(29), 
    Some(20), Some(20), Some(20), Some(20), None, None, Some(20), Some(20), Some(20), Some(20), Some(20), 
    Some(20), Some(20), Some(20), Some(20), Some(20), Some(20), Some(20), Some(20), Some(20), Some(20), 
    Some(88), Some(88), Some(88), Some(88), None, None, Some(88), Some(88), Some(88), Some(88), Some(88), 
    Some(88), Some(88), Some(88), Some(88), Some(88), Some(88), Some(88), Some(88), Some(88), Some(88), 
    Some(83), Some(83), Some(83), Some(83), None, None, Some(83), Some(83), Some(83), Some(83), Some(83), 
    Some(83), Some(83), Some(83), Some(83), Some(83), Some(83), Some(83), Some(83), Some(83), None, None, 
    None, Some(83), Some(76), Some(76), Some(76), Some(76), None, None, Some(76), Some(76), Some(76), 
    Some(76), Some(76), Some(76), Some(76), Some(76), Some(76), Some(76), Some(76), Some(76), Some(76), 
    Some(76), Some(76), Some(69), Some(69), Some(69), Some(69), None, None, Some(69), Some(69), Some(69), 
    Some(69), Some(69), Some(69), Some(69), Some(69), Some(69), Some(69), Some(69), Some(69), Some(69), 
    Some(69), None, None, None, Some(69), Some(49), Some(49), Some(49), Some(49), None, None, Some(49), 
    Some(49), Some(49), Some(49), Some(49), Some(49), Some(49), Some(49), Some(49), Some(49), Some(49), 
    Some(49), Some(49), Some(49), Some(49), Some(42), Some(42), Some(42), Some(42), None, None, Some(42), 
    Some(42), Some(42), Some(42), Some(42), Some(42), Some(42), Some(42), Some(42), Some(42), Some(42), 
    Some(42), Some(42), Some(42), None, None, None, Some(42), Some(37), Some(37), Some(37), Some(37), 
    None, None, Some(37), Some(37), Some(37), Some(37), Some(37), Some(37), Some(37), Some(37), Some(37), 
    Some(37), Some(37), Some(37), Some(37), Some(37), None, None, None, Some(37), Some(89), Some(89), 
    Some(89), Some(89), None, None, Some(89), Some(89), Some(89), Some(89), Some(89), Some(89), Some(89), 
    Some(89), Some(89), Some(89), Some(89), Some(89), Some(89), Some(89), Some(68), Some(68), Some(68), 
    Some(68), None, None, Some(68), Some(68), Some(68), Some(68), Some(68), Some(68), Some(68), Some(68), 
    Some(68), Some(68), Some(68), Some(68), Some(85), None, Some(85), Some(85), Some(85), Some(85), Some(85), 
    Some(7), Some(7), Some(7), Some(7), Some(7), Some(7), Some(7), Some(85), None, None, Some(85), Some(85), 
    Some(85), None, Some(7), None, None, None, Some(85), Some(85), Some(85), None, Some(85), None, Some(85), 
    Some(66), Some(66), Some(66), Some(66), None, None, Some(66), Some(66), Some(66), Some(66), Some(66), 
    Some(66), Some(66), Some(66), Some(66), Some(66), Some(66), Some(87), None, Some(87), Some(87), Some(87), 
    Some(87), Some(87), None, None, None, Some(22), Some(22), Some(22), Some(22), Some(87), Some(22), 
    Some(22), Some(22), Some(87), Some(87), None, Some(22), None, None, None, Some(87), Some(87), Some(87), 
    None, Some(87), None, Some(87), Some(65), Some(65), Some(65), Some(65), None, None, Some(65), Some(65), 
    Some(65), Some(65), Some(65), Some(65), Some(65), Some(65), Some(65), Some(65), Some(0), Some(0), 
    Some(0), None, None, Some(0), None, None, None, Some(0), None, Some(0), Some(0), Some(0), Some(0), 
    Some(0), None, None, Some(28), Some(28), Some(28), Some(28), None, Some(0), Some(28), Some(28), Some(0), 
    Some(0), Some(0), Some(64), Some(64), Some(64), Some(64), None, None, Some(64), Some(64), Some(64), 
    Some(64), Some(64), Some(64), Some(64), Some(64), Some(64), Some(4), Some(4), None, None, Some(4), 
    Some(41), Some(41), Some(41), Some(4), None, Some(4), Some(4), Some(4), Some(4), Some(4), None, Some(41), 
    Some(41), None, None, None, None, Some(4), None, None, Some(4), Some(4), Some(4), Some(63), Some(63), 
    Some(63), Some(63), None, None, Some(63), Some(63), Some(63), Some(63), Some(63), Some(63), Some(63), 
    Some(63), Some(24), None, Some(24), Some(24), Some(24), Some(24), Some(24), Some(95), None, Some(95), 
    Some(95), Some(95), Some(95), Some(95), Some(24), Some(5), Some(5), Some(5), None, None, None, Some(95), 
    None, Some(24), Some(24), Some(24), Some(24), Some(24), None, None, None, None, Some(95), Some(95), 
    Some(95), Some(5), Some(5), Some(5), Some(95), Some(62), Some(62), Some(62), Some(62), None, None, 
    Some(62), Some(62), Some(62), Some(62), Some(62), Some(62), Some(62), Some(61), Some(61), Some(61), 
    Some(61), None, None, Some(61), Some(61), Some(61), Some(61), Some(61), Some(61), Some(61), Some(60), 
    Some(60), Some(60), Some(60), None, None, Some(60), Some(60), Some(60), Some(60), Some(60), Some(60), 
    Some(59), Some(59), Some(59), Some(59), None, None, Some(59), Some(59), Some(59), Some(59), Some(59), 
    Some(59), Some(58), Some(58), Some(58), Some(58), None, None, Some(58), Some(58), Some(58), Some(58), 
    Some(58), Some(58), Some(57), Some(57), Some(57), Some(57), None, None, Some(57), Some(57), Some(57), 
    Some(57), Some(57), Some(57), Some(10), Some(10), Some(10), None, None, None, Some(10), None, Some(10), 
    Some(10), Some(10), Some(10), Some(10), None, None, Some(56), Some(56), Some(56), Some(56), None, 
    Some(10), Some(56), Some(56), Some(56), Some(56), Some(56), Some(55), Some(55), Some(55), Some(55), 
    None, None, Some(55), Some(55), Some(55), Some(55), Some(55), Some(78), None, Some(78), Some(78), 
    Some(78), Some(78), Some(78), None, None, Some(54), Some(54), Some(54), Some(54), None, Some(78), 
    Some(54), Some(54), Some(54), Some(54), None, None, None, Some(53), Some(53), Some(53), Some(53), 
    None, Some(78), Some(53), Some(53), Some(53), Some(53), Some(15), None, None, None, Some(15), None, 
    Some(15), Some(15), Some(15), Some(15), Some(15), Some(6), Some(6), Some(6), Some(6), Some(6), Some(6), 
    Some(6), Some(15), None, Some(80), Some(80), Some(80), Some(80), None, Some(6), Some(80), Some(80), 
    Some(80), Some(52), Some(52), Some(52), Some(52), None, None, Some(52), Some(52), Some(52), Some(51), 
    Some(51), Some(51), Some(51), None, None, Some(51), Some(51), Some(51), Some(50), Some(50), Some(50), 
    Some(50), None, None, Some(50), Some(50), Some(50), Some(34), Some(34), Some(34), Some(34), None, 
    None, None, Some(34), Some(34), None, None, Some(34), Some(33), Some(33), Some(33), Some(33), None, 
    None, None, Some(33), Some(33), None, None, Some(33), Some(32), Some(32), Some(32), Some(32), Some(36), 
    Some(36), Some(32), Some(32), Some(32), Some(36), Some(36), Some(36), 
];

static GOTO_ROW_ID: [usize; 348] = [
    0, 1, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 3, 4, 2, 2, 2, 5, 6, 7, 8, 2, 9, 2, 
    10, 11, 2, 2, 2, 12, 2, 2, 2, 2, 2, 2, 13, 2, 2, 2, 2, 2, 2, 14, 15, 2, 16, 17, 2, 2, 2, 2, 18, 2, 
    19, 20, 21, 22, 23, 2, 2, 2, 2, 24, 25, 2, 2, 2, 2, 2, 2, 26, 2, 2, 2, 27, 2, 2, 2, 2, 2, 2, 28, 
    29, 30, 2, 2, 2, 2, 2, 31, 32, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 33, 34, 35, 2, 36, 2, 2, 
    2, 37, 2, 2, 2, 2, 38, 2, 2, 2, 2, 39, 2, 2, 2, 2, 40, 2, 2, 2, 2, 2, 2, 2, 2, 2, 41, 2, 2, 2, 2, 
    2, 2, 2, 2, 2, 42, 43, 2, 44, 2, 2, 2, 2, 45, 2, 46, 2, 2, 2, 47, 2, 48, 2, 2, 2, 2, 2, 2, 2, 2, 
    2, 2, 2, 2, 2, 2, 49, 2, 50, 51, 52, 53, 54, 55, 56, 57, 58, 59, 60, 61, 62, 63, 64, 65, 66, 67, 
    68, 2, 2, 2, 2, 2, 69, 2, 2, 2, 70, 2, 71, 72, 73, 74, 2, 75, 2, 2, 2, 2, 2, 2, 2, 76, 2, 2, 77, 
    2, 2, 2, 2, 2, 2, 78, 2, 2, 2, 79, 2, 2, 80, 81, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 
    2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 82, 2, 83, 2, 84, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 85, 86, 2, 
    2, 2, 2, 87, 2, 2, 2, 2, 2, 88, 2, 89, 2, 2, 2, 90, 91, 92, 93, 94, 2, 2, 2, 2, 95, 96, 2, 2, 2, 
    2, 2, 2, 2, 2, 2, 2, 2, 2, 97, 2, 98, 2, 2, 2, 2, 2, 99, 2, 
];


/// prod_id -> length for reduce
pub static EXPR_LENS: [usize; 218] = [
    1, 2, 1, 1, 4, 3, 0, 1, 1, 2, 3, 0, 1, 1, 3, 1, 3, 2, 2, 2, 0, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 
    1, 1, 1, 1, 1, 1, 1, 1, 5, 2, 1, 1, 0, 1, 1, 2, 3, 2, 2, 0, 1, 1, 3, 1, 2, 3, 5, 2, 1, 3, 1, 3, 2, 
    0, 1, 1, 2, 2, 3, 1, 2, 1, 3, 4, 4, 4, 0, 1, 0, 1, 1, 3, 1, 3, 1, 3, 2, 2, 0, 1, 1, 2, 3, 3, 4, 3, 
    4, 0, 1, 1, 3, 4, 1, 3, 1, 1, 1, 1, 1, 1, 3, 4, 3, 2, 3, 1, 2, 1, 1, 1, 2, 5, 7, 5, 5, 7, 9, 0, 1, 
    3, 2, 2, 2, 3, 1, 1, 1, 3, 1, 1, 1, 1, 2, 1, 4, 4, 3, 3, 2, 2, 0, 1, 1, 3, 1, 2, 2, 2, 2, 4, 1, 1, 
    1, 1, 1, 1, 4, 1, 3, 3, 3, 1, 3, 3, 1, 3, 3, 1, 3, 3, 3, 3, 1, 3, 3, 1, 3, 1, 3, 1, 3, 1, 3, 1, 3, 
    1, 1, 5, 1, 3, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 3, 1, 2, 
];

/// prod_id -> expr name(non-terminal name)
pub static EXPR_NAMES: [&str; 218] = [
    "translation_unit", "translation_unit", "external_declaration", "external_declaration", "function_definition", 
    "function_definition", "declaration_list_opt", "declaration_list_opt", "declaration_list", "declaration_list", 
    "declaration", "init_declarator_list_opt", "init_declarator_list_opt", "init_declarator_list", "init_declarator_list", 
    "init_declarator", "init_declarator", "declaration_specifiers", "declaration_specifiers", "declaration_specifiers", 
    "declaration_specifiers_opt", "declaration_specifiers_opt", "storage_class_specifier", "storage_class_specifier", 
    "storage_class_specifier", "storage_class_specifier", "storage_class_specifier", "type_specifier", 
    "type_specifier", "type_specifier", "type_specifier", "type_specifier", "type_specifier", "type_specifier", 
    "type_specifier", "type_specifier", "type_specifier", "type_specifier", "type_specifier", "type_qualifier", 
    "type_qualifier", "struct_or_union_specifier", "struct_or_union_specifier", "struct_or_union", "struct_or_union", 
    "identifier_opt", "identifier_opt", "struct_declaration_list", "struct_declaration_list", "struct_declaration", 
    "specifier_qualifier_list", "specifier_qualifier_list", "specifier_qualifier_list_opt", "specifier_qualifier_list_opt", 
    "struct_declarator_list", "struct_declarator_list", "struct_declarator", "struct_declarator", "struct_declarator", 
    "enum_specifier", "enum_specifier", "enumerator_list", "enumerator_list", "enumerator", "enumerator", 
    "declarator", "pointer_opt", "pointer_opt", "pointer", "pointer", "pointer", "pointer", "type_qualifier_list", 
    "type_qualifier_list", "direct_declarator", "direct_declarator", "direct_declarator", "direct_declarator", 
    "direct_declarator", "constant_expression_opt", "constant_expression_opt", "identifier_list_opt", 
    "identifier_list_opt", "identifier_list", "identifier_list", "parameter_type_list", "parameter_type_list", 
    "parameter_list", "parameter_list", "parameter_declaration", "parameter_declaration", "abstract_declarator_opt", 
    "abstract_declarator_opt", "abstract_declarator", "abstract_declarator", "direct_abstract_declarator", 
    "direct_abstract_declarator", "direct_abstract_declarator", "direct_abstract_declarator", "direct_abstract_declarator", 
    "parameter_type_list_opt", "parameter_type_list_opt", "initializer", "initializer", "initializer", 
    "initializer_list", "initializer_list", "statement", "statement", "statement", "statement", "statement", 
    "statement", "labeled_statement", "labeled_statement", "labeled_statement", "compound_statement", 
    "compound_statement", "block_item_list", "block_item_list", "block_item", "block_item", "expression_statement", 
    "expression_statement", "selection_statement", "selection_statement", "selection_statement", "iteration_statement", 
    "iteration_statement", "iteration_statement", "expression_opt", "expression_opt", "jump_statement", 
    "jump_statement", "jump_statement", "jump_statement", "jump_statement", "primary_expression", "primary_expression", 
    "primary_expression", "primary_expression", "constant", "constant", "constant", "string", "string", 
    "postfix_expression", "postfix_expression", "postfix_expression", "postfix_expression", "postfix_expression", 
    "postfix_expression", "postfix_expression", "argument_expression_list_opt", "argument_expression_list_opt", 
    "argument_expression_list", "argument_expression_list", "unary_expression", "unary_expression", "unary_expression", 
    "unary_expression", "unary_expression", "unary_expression", "unary_operator", "unary_operator", "unary_operator", 
    "unary_operator", "unary_operator", "unary_operator", "cast_expression", "cast_expression", "multiplicative_expression", 
    "multiplicative_expression", "multiplicative_expression", "multiplicative_expression", "additive_expression", 
    "additive_expression", "additive_expression", "shift_expression", "shift_expression", "shift_expression", 
    "relational_expression", "relational_expression", "relational_expression", "relational_expression", 
    "relational_expression", "equality_expression", "equality_expression", "equality_expression", "and_expression", 
    "and_expression", "exclusive_or_expression", "exclusive_or_expression", "inclusive_or_expression", 
    "inclusive_or_expression", "logical_and_expression", "logical_and_expression", "logical_or_expression", 
    "logical_or_expression", "conditional_expression", "conditional_expression", "assignment_expression", 
    "assignment_expression", "assignment_operator", "assignment_operator", "assignment_operator", "assignment_operator", 
    "assignment_operator", "assignment_operator", "assignment_operator", "assignment_operator", "assignment_operator", 
    "assignment_operator", "assignment_operator", "expression", "expression", "constant_expression", 
    "type_name", 
];

/// prod_id -> rule id(non-terminal name)
static EXPR_IDS: [usize; 218] = [
    0, 0, 1, 1, 2, 2, 3, 3, 4, 4, 5, 6, 6, 7, 7, 8, 8, 9, 9, 9, 10, 10, 11, 11, 11, 11, 11, 12, 12, 12, 
    12, 12, 12, 12, 12, 12, 12, 12, 12, 13, 13, 14, 14, 15, 15, 16, 16, 17, 17, 18, 19, 19, 20, 20, 21, 
    21, 22, 22, 22, 23, 23, 24, 24, 25, 25, 26, 27, 27, 28, 28, 28, 28, 29, 29, 30, 30, 30, 30, 30, 31, 
    31, 32, 32, 33, 33, 34, 34, 35, 35, 36, 36, 37, 37, 38, 38, 39, 39, 39, 39, 39, 40, 40, 41, 41, 41, 
    42, 42, 43, 43, 43, 43, 43, 43, 44, 44, 44, 45, 45, 46, 46, 47, 47, 48, 48, 49, 49, 49, 50, 50, 50, 
    51, 51, 52, 52, 52, 52, 52, 53, 53, 53, 53, 54, 54, 54, 55, 55, 56, 56, 56, 56, 56, 56, 56, 57, 57, 
    58, 58, 59, 59, 59, 59, 59, 59, 60, 60, 60, 60, 60, 60, 61, 61, 62, 62, 62, 62, 63, 63, 63, 64, 64, 
    64, 65, 65, 65, 65, 65, 66, 66, 66, 67, 67, 68, 68, 69, 69, 70, 70, 71, 71, 72, 72, 73, 73, 74, 74, 
    74, 74, 74, 74, 74, 74, 74, 74, 74, 75, 75, 76, 77, 
];

/// token_id -> token content (terminal name)
pub static TOKEN_CONTENTS: [Option<&str>; 318] = [
    None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, 
    None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, Some("'!'"), 
    None, None, None, Some("'%'"), Some("'&'"), None, Some("'('"), Some("')'"), Some("'*'"), Some("'+'"), 
    Some("','"), Some("'-'"), Some("'.'"), Some("'/'"), None, None, None, None, None, None, None, None, 
    None, None, Some("':'"), Some("';'"), Some("'<'"), Some("'='"), Some("'>'"), Some("'?'"), None, None, 
    None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, 
    None, None, None, None, None, None, None, None, Some("'['"), None, Some("']'"), Some("'^'"), None, 
    None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, 
    None, None, None, None, None, None, None, None, None, None, Some("'{'"), Some("'|'"), Some("'}'"), 
    Some("'~'"), None, None, None, None, None, None, None, None, None, None, None, None, None, None, 
    None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, 
    None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, 
    None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, 
    None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, 
    None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, 
    None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, 
    None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, Some("ID"), 
    Some("TYPE_NAME"), Some("INT"), Some("FLOAT"), Some("CHARACTER_CONSTANT"), Some("STRING_LITERAL"), 
    Some("KEYWORD_SIZEOF"), Some("OP_ARROW"), Some("OP_INC"), Some("OP_DEC"), Some("OP_L_SHIFT"), Some("OP_R_SHIFT"), 
    Some("OP_LE"), Some("OP_GE"), Some("OP_EQ"), Some("OP_NE"), Some("OP_AND"), Some("OP_OR"), Some("OP_MUL_ASSIGN"), 
    Some("OP_DIV_ASSIGN"), Some("OP_MOD_ASSIGN"), Some("OP_ADD_ASSIGN"), Some("OP_SUB_ASSIGN"), Some("OP_L_SHIFT_ASSIGN"), 
    Some("OP_R_SHIFT_ASSIGN"), Some("OP_AND_ASSIGN"), Some("OP_XOR_ASSIGN"), Some("OP_OR_ASSIGN"), Some("KEYWORD_TYPEDEF"), 
    Some("KEYWORD_EXTERN"), Some("KEYWORD_STATIC"), Some("KEYWORD_AUTO"), Some("KEYWORD_REGISTER"), Some("KEYWORD_CHAR"), 
    Some("KEYWORD_SHORT"), Some("KEYWORD_INT"), Some("KEYWORD_LONG"), Some("KEYWORD_SIGNED"), Some("KEYWORD_UNSIGNED"), 
    Some("KEYWORD_FLOAT"), Some("KEYWORD_DOUBLE"), Some("KEYWORD_VOID"), Some("KEYWORD_CONST"), Some("KEYWORD_VOLATILE"), 
    Some("KEYWORD_STRUCT"), Some("KEYWORD_UNION"), Some("KEYWORD_ENUM"), Some("KEYWORD_CASE"), Some("KEYWORD_DEFAULT"), 
    Some("KEYWORD_IF"), Some("KEYWORD_ELSE"), Some("KEYWORD_SWITCH"), Some("KEYWORD_WHILE"), Some("KEYWORD_DO"), 
    Some("KEYWORD_FOR"), Some("KEYWORD_GOTO"), Some("KEYWORD_CONTINUE"), Some("KEYWORD_BREAK"), Some("KEYWORD_RETURN"), 
    Some("OP_ELLIPSIS"), 
];type ActionHandler = fn(_arguments: Vec<ParserNode>,) -> ParserNode;

pub static ACTION_CODES: [Option<ActionHandler>;218] = [
    Some(|_arguments: Vec<ParserNode>,| {     // 0
        let value;
        destruct_vec!(_arguments, _arg1);
        value = TranslationUnit::make_translation_unit(_arg1.into());
        value
    }),
    Some(|_arguments: Vec<ParserNode>,| {     // 1
        let value;
        destruct_vec!(_arguments, _arg1, _arg2);
        value = TranslationUnit::insert_ext_decl(_arg1.into(), _arg2.into());
        value
    }),
    Some(|_arguments: Vec<ParserNode>,| {     // 2
        let value;
        destruct_vec!(_arguments, _arg1);
        value = ExternalDeclaration::make_func(_arg1.into());
        value
    }),
    Some(|_arguments: Vec<ParserNode>,| {     // 3
        let value;
        destruct_vec!(_arguments, _arg1);
        value = ExternalDeclaration::make_decl(_arg1.into());
        value
    }),
    Some(|_arguments: Vec<ParserNode>,| {     // 4
        let value;
        destruct_vec!(_arguments, _arg1, _arg2, _arg3, _arg4);
         value = FunctionDefinition::make(Some(_arg1.into()), _arg2.into(), _arg3.into(), _arg4.into()); 
        value
    }),
    Some(|_arguments: Vec<ParserNode>,| {     // 5
        let value;
        destruct_vec!(_arguments, _arg1, _arg2, _arg3);
         value = FunctionDefinition::make(None, _arg1.into(), _arg2.into(), _arg3.into()); 
        value
    }),
    Some(|_arguments: Vec<ParserNode>,| {     // 6
        let value;
         value = ParserNode::None; 
        value
    }),
    Some(|_arguments: Vec<ParserNode>,| {     // 7
        let value;
        destruct_vec!(_arguments, _arg1);
         value = _arg1.into(); 
        value
    }),
    Some(|_arguments: Vec<ParserNode>,| {     // 8
        let value;
        destruct_vec!(_arguments, _arg1);
         value = Decl::make_list(_arg1.into()); 
        value
    }),
    Some(|_arguments: Vec<ParserNode>,| {     // 9
        let value;
        destruct_vec!(_arguments, _arg1, _arg2);
         value = Decl::push(_arg1.into(), _arg2.into()); 
        value
    }),
    Some(|_arguments: Vec<ParserNode>,| {     // 10
        let value;
        destruct_vec!(_arguments, _arg1, _arg2, _arg3);
         value = Decl::make(_arg1.into(), _arg2.into(), _arg3.into()); 
        value
    }),
    Some(|_arguments: Vec<ParserNode>,| {     // 11
        let value;
        value = ParserNode::None;
        value
    }),
    Some(|_arguments: Vec<ParserNode>,| {     // 12
        let value;
        destruct_vec!(_arguments, _arg1);
        value = _arg1.into();
        value
    }),
    Some(|_arguments: Vec<ParserNode>,| {     // 13
        let value;
        destruct_vec!(_arguments, _arg1);
        value = InitDeclarator::make_list(_arg1.into()); 
        value
    }),
    Some(|_arguments: Vec<ParserNode>,| {     // 14
        let value;
        destruct_vec!(_arguments, _arg1, _arg2, _arg3);
        value = InitDeclarator::push(_arg1.into(), _arg2.into(), _arg3.into()); 
        value
    }),
    Some(|_arguments: Vec<ParserNode>,| {     // 15
        let value;
        destruct_vec!(_arguments, _arg1);
        value = InitDeclarator::make(_arg1.into(), None, None); 
        value
    }),
    Some(|_arguments: Vec<ParserNode>,| {     // 16
        let value;
        destruct_vec!(_arguments, _arg1, _arg2, _arg3);
        value = InitDeclarator::make(_arg1.into(), Some(_arg2.into()), Some(_arg3.into())); 
        value
    }),
    Some(|_arguments: Vec<ParserNode>,| {     // 17
        let value;
        destruct_vec!(_arguments, _arg1, _arg2);
        value = DeclSpec::push_storage(_arg1.into(), _arg2.into());
        value
    }),
    Some(|_arguments: Vec<ParserNode>,| {     // 18
        let value;
        destruct_vec!(_arguments, _arg1, _arg2);
        value = DeclSpec::push_spec(_arg1.into(), _arg2.into());
        value
    }),
    Some(|_arguments: Vec<ParserNode>,| {     // 19
        let value;
        destruct_vec!(_arguments, _arg1, _arg2);
        value = DeclSpec::push_qual(_arg1.into(), _arg2.into());
        value
    }),
    Some(|_arguments: Vec<ParserNode>,| {     // 20
        let value;
        value = ParserNode::None;
        value
    }),
    Some(|_arguments: Vec<ParserNode>,| {     // 21
        let value;
        destruct_vec!(_arguments, _arg1);
        value = _arg1.into();
        value
    }),
    Some(|_arguments: Vec<ParserNode>,| {     // 22
        let value;
        destruct_vec!(_arguments, _arg1);
        value = _arg1.into();
        value
    }),
    Some(|_arguments: Vec<ParserNode>,| {     // 23
        let value;
        destruct_vec!(_arguments, _arg1);
        value = _arg1.into();
        value
    }),
    Some(|_arguments: Vec<ParserNode>,| {     // 24
        let value;
        destruct_vec!(_arguments, _arg1);
        value = _arg1.into();
        value
    }),
    Some(|_arguments: Vec<ParserNode>,| {     // 25
        let value;
        destruct_vec!(_arguments, _arg1);
        value = _arg1.into();
        value
    }),
    Some(|_arguments: Vec<ParserNode>,| {     // 26
        let value;
        destruct_vec!(_arguments, _arg1);
        value = _arg1.into();
        value
    }),
    Some(|_arguments: Vec<ParserNode>,| {     // 27
        let value;
        destruct_vec!(_arguments, _arg1);
        value = TypeSpec::make(_arg1.into());
        value
    }),
    Some(|_arguments: Vec<ParserNode>,| {     // 28
        let value;
        destruct_vec!(_arguments, _arg1);
        value = TypeSpec::make(_arg1.into());
        value
    }),
    Some(|_arguments: Vec<ParserNode>,| {     // 29
        let value;
        destruct_vec!(_arguments, _arg1);
        value = TypeSpec::make(_arg1.into());
        value
    }),
    Some(|_arguments: Vec<ParserNode>,| {     // 30
        let value;
        destruct_vec!(_arguments, _arg1);
        value = TypeSpec::make(_arg1.into());
        value
    }),
    Some(|_arguments: Vec<ParserNode>,| {     // 31
        let value;
        destruct_vec!(_arguments, _arg1);
        value = TypeSpec::make(_arg1.into());
        value
    }),
    Some(|_arguments: Vec<ParserNode>,| {     // 32
        let value;
        destruct_vec!(_arguments, _arg1);
        value = TypeSpec::make(_arg1.into());
        value
    }),
    Some(|_arguments: Vec<ParserNode>,| {     // 33
        let value;
        destruct_vec!(_arguments, _arg1);
        value = TypeSpec::make(_arg1.into());
        value
    }),
    Some(|_arguments: Vec<ParserNode>,| {     // 34
        let value;
        destruct_vec!(_arguments, _arg1);
        value = TypeSpec::make(_arg1.into());
        value
    }),
    Some(|_arguments: Vec<ParserNode>,| {     // 35
        let value;
        destruct_vec!(_arguments, _arg1);
        value = TypeSpec::make(_arg1.into());
        value
    }),
    Some(|_arguments: Vec<ParserNode>,| {     // 36
        let value;
        destruct_vec!(_arguments, _arg1);
        value = TypeSpec::make_struct_or_union(_arg1.into());
        value
    }),
    Some(|_arguments: Vec<ParserNode>,| {     // 37
        let value;
        destruct_vec!(_arguments, _arg1);
        value = TypeSpec::make_enum(_arg1.into());
        value
    }),
    Some(|_arguments: Vec<ParserNode>,| {     // 38
        let value;
        destruct_vec!(_arguments, _arg1);
        value = TypeSpec::make(_arg1.into());
        value
    }),
    Some(|_arguments: Vec<ParserNode>,| {     // 39
        let value;
        destruct_vec!(_arguments, _arg1);
        value = _arg1.into();
        value
    }),
    Some(|_arguments: Vec<ParserNode>,| {     // 40
        let value;
        destruct_vec!(_arguments, _arg1);
        value = _arg1.into();
        value
    }),
    Some(|_arguments: Vec<ParserNode>,| {     // 41
        let value;
        destruct_vec!(_arguments, _arg1, _arg2, _arg3, _arg4, _arg5);
        value = StructUnionSpec::make_def(_arg1.into(), _arg2.into(), _arg3.into(), _arg4.into(), _arg5.into());
        value
    }),
    Some(|_arguments: Vec<ParserNode>,| {     // 42
        let value;
        destruct_vec!(_arguments, _arg1, _arg2);
        value = StructUnionSpec::make_decl(_arg1.into(), _arg2.into());
        value
    }),
    Some(|_arguments: Vec<ParserNode>,| {     // 43
        let value;
        destruct_vec!(_arguments, _arg1);
        value = _arg1.into();
        value
    }),
    Some(|_arguments: Vec<ParserNode>,| {     // 44
        let value;
        destruct_vec!(_arguments, _arg1);
        value = _arg1.into();
        value
    }),
    Some(|_arguments: Vec<ParserNode>,| {     // 45
        let value;
        value = ParserNode::None;
        value
    }),
    Some(|_arguments: Vec<ParserNode>,| {     // 46
        let value;
        destruct_vec!(_arguments, _arg1);
        value = _arg1.into();
        value
    }),
    Some(|_arguments: Vec<ParserNode>,| {     // 47
        let value;
        destruct_vec!(_arguments, _arg1);
        value = StructDecl::make_list(_arg1.into());
        value
    }),
    Some(|_arguments: Vec<ParserNode>,| {     // 48
        let value;
        destruct_vec!(_arguments, _arg1, _arg2);
        value = StructDecl::push(_arg1.into(), _arg2.into());
        value
    }),
    Some(|_arguments: Vec<ParserNode>,| {     // 49
        let value;
        destruct_vec!(_arguments, _arg1, _arg2, _arg3);
        value = StructDecl::make(_arg1.into(), _arg2.into(), _arg3.into()); 
        value
    }),
    Some(|_arguments: Vec<ParserNode>,| {     // 50
        let value;
        destruct_vec!(_arguments, _arg1, _arg2);
        value = DeclSpec::push_spec(_arg1.into(), _arg2.into()); 
        value
    }),
    Some(|_arguments: Vec<ParserNode>,| {     // 51
        let value;
        destruct_vec!(_arguments, _arg1, _arg2);
        value = DeclSpec::push_qual(_arg1.into(), _arg2.into()); 
        value
    }),
    Some(|_arguments: Vec<ParserNode>,| {     // 52
        let value;
        value = ParserNode::None;
        value
    }),
    Some(|_arguments: Vec<ParserNode>,| {     // 53
        let value;
        destruct_vec!(_arguments, _arg1);
        value = _arg1.into();
        value
    }),
    Some(|_arguments: Vec<ParserNode>,| {     // 54
        let value;
        destruct_vec!(_arguments, _arg1);
        value = StructDeclarator::make_list(_arg1.into()); 
        value
    }),
    Some(|_arguments: Vec<ParserNode>,| {     // 55
        let value;
        destruct_vec!(_arguments, _arg1, _arg2, _arg3);
        value = StructDeclarator::push(_arg1.into(), _arg2.into(), _arg3.into()); 
        value
    }),
    Some(|_arguments: Vec<ParserNode>,| {     // 56
        let value;
        destruct_vec!(_arguments, _arg1);
        value = StructDeclarator::make(Some(_arg1.into()), None, None); 
        value
    }),
    Some(|_arguments: Vec<ParserNode>,| {     // 57
        let value;
        destruct_vec!(_arguments, _arg1, _arg2);
        value = StructDeclarator::make(None, Some(_arg1.into()), Some(_arg2.into())); 
        value
    }),
    Some(|_arguments: Vec<ParserNode>,| {     // 58
        let value;
        destruct_vec!(_arguments, _arg1, _arg2, _arg3);
        value = StructDeclarator::make(Some(_arg1.into()), Some(_arg2.into()), Some(_arg3.into())); 
        value
    }),
    Some(|_arguments: Vec<ParserNode>,| {     // 59
        let value;
        destruct_vec!(_arguments, _arg1, _arg2, _arg3, _arg4, _arg5);
        value = EnumSpec::make_anon(_arg1.into(), _arg2.into(), _arg3.into(), _arg4.into(), _arg5.into());
        value
    }),
    Some(|_arguments: Vec<ParserNode>,| {     // 60
        let value;
        destruct_vec!(_arguments, _arg1, _arg2);
        value = EnumSpec::make_named(_arg1.into(), _arg2.into());
        value
    }),
    Some(|_arguments: Vec<ParserNode>,| {     // 61
        let value;
        destruct_vec!(_arguments, _arg1);
        value = Enumerator::make_list(_arg1.into());
        value
    }),
    Some(|_arguments: Vec<ParserNode>,| {     // 62
        let value;
        destruct_vec!(_arguments, _arg1, _arg2, _arg3);
        value = Enumerator::push(_arg1.into(), _arg2.into(), _arg3.into());
        value
    }),
    Some(|_arguments: Vec<ParserNode>,| {     // 63
        let value;
        destruct_vec!(_arguments, _arg1);
        value = Enumerator::make(_arg1.into(), None);
        value
    }),
    Some(|_arguments: Vec<ParserNode>,| {     // 64
        let value;
        destruct_vec!(_arguments, _arg1, _arg2, _arg3);
        value = Enumerator::make(_arg1.into(), Some(_arg3.into()));
        value
    }),
    Some(|_arguments: Vec<ParserNode>,| {     // 65
        let value;
        destruct_vec!(_arguments, _arg1, _arg2);
        value = Declarator::make(_arg1.into(), Some(_arg2.into()));
        value
    }),
    Some(|_arguments: Vec<ParserNode>,| {     // 66
        let value;
        value = ParserNode::None;
        value
    }),
    Some(|_arguments: Vec<ParserNode>,| {     // 67
        let value;
        destruct_vec!(_arguments, _arg1);
        value = _arg1.into();
        value
    }),
    Some(|_arguments: Vec<ParserNode>,| {     // 68
        let value;
        destruct_vec!(_arguments, _arg1);
        value = PointerChunk::make_list( PointerChunk::make_pointer(_arg1.into(), None) );
        value
    }),
    Some(|_arguments: Vec<ParserNode>,| {     // 69
        let value;
        destruct_vec!(_arguments, _arg1, _arg2);
        value = PointerChunk::make_list( PointerChunk::make_pointer(_arg1.into(), _arg2.into()) );
        value
    }),
    Some(|_arguments: Vec<ParserNode>,| {     // 70
        let value;
        destruct_vec!(_arguments, _arg1, _arg2);
        value = PointerChunk::push_front( PointerChunk::make_pointer(_arg1.into(), None), _arg2.into() );
        value
    }),
    Some(|_arguments: Vec<ParserNode>,| {     // 71
        let value;
        destruct_vec!(_arguments, _arg1, _arg2, _arg3);
        value = PointerChunk::push_front( PointerChunk::make_pointer(_arg1.into(), _arg2.into()), _arg3.into() );
        value
    }),
    Some(|_arguments: Vec<ParserNode>,| {     // 72
        let value;
        destruct_vec!(_arguments, _arg1);
        value = TypeQual::make(None, _arg1.into());
        value
    }),
    Some(|_arguments: Vec<ParserNode>,| {     // 73
        let value;
        destruct_vec!(_arguments, _arg1, _arg2);
        value = TypeQual::make(Some(_arg1.into()), _arg2.into());
        value
    }),
    Some(|_arguments: Vec<ParserNode>,| {     // 74
        let value;
        destruct_vec!(_arguments, _arg1);
        value = DeclChunk::make_list( DeclChunk::make_ident(_arg1.into()) ); 
        value
    }),
    Some(|_arguments: Vec<ParserNode>,| {     // 75
        let value;
        destruct_vec!(_arguments, _arg1, _arg2, _arg3);
        value = DeclChunk::make_list( DeclChunk::make_paren(_arg1.into(), _arg2.into(), _arg3.into()) ); 
        value
    }),
    Some(|_arguments: Vec<ParserNode>,| {     // 76
        let value;
        destruct_vec!(_arguments, _arg1, _arg2, _arg3, _arg4);
        value = DeclChunk::push( _arg1.into(), DeclChunk::make_array(_arg2.into(), _arg3.into(), _arg4.into()) ); 
        value
    }),
    Some(|_arguments: Vec<ParserNode>,| {     // 77
        let value;
        destruct_vec!(_arguments, _arg1, _arg2, _arg3, _arg4);
        value = DeclChunk::push( _arg1.into(), DeclChunk::make_function(_arg2.into(), _arg3.into(), _arg4.into()) );
        value
    }),
    Some(|_arguments: Vec<ParserNode>,| {     // 78
        let value;
        destruct_vec!(_arguments, _arg1, _arg2, _arg3, _arg4);
        value = DeclChunk::push( _arg1.into(), DeclChunk::make_kr_function(_arg2.into(), _arg3.into(), _arg4.into()) ); 
        value
    }),
    Some(|_arguments: Vec<ParserNode>,| {     // 79
        let value;
        value = ParserNode::None;
        value
    }),
    Some(|_arguments: Vec<ParserNode>,| {     // 80
        let value;
        destruct_vec!(_arguments, _arg1);
        value = _arg1.into();
        value
    }),
    Some(|_arguments: Vec<ParserNode>,| {     // 81
        let value;
        value = ParserNode::None;
        value
    }),
    Some(|_arguments: Vec<ParserNode>,| {     // 82
        let value;
        destruct_vec!(_arguments, _arg1);
        value = _arg1.into();
        value
    }),
    Some(|_arguments: Vec<ParserNode>,| {     // 83
        let value;
        destruct_vec!(_arguments, _arg1);
        value = make_ident_list(_arg1.into());
        value
    }),
    Some(|_arguments: Vec<ParserNode>,| {     // 84
        let value;
        destruct_vec!(_arguments, _arg1, _arg2, _arg3);
        value = push_ident_list(_arg1.into(), _arg2.into(), _arg3.into());
        value
    }),
    Some(|_arguments: Vec<ParserNode>,| {     // 85
        let value;
        destruct_vec!(_arguments, _arg1);
        value = _arg1.into();
        value
    }),
    Some(|_arguments: Vec<ParserNode>,| {     // 86
        let value;
        destruct_vec!(_arguments, _arg1, _arg2, _arg3);
        value = ParamList::set_variadic(_arg1.into(), _arg2.into(), _arg3.into());
        value
    }),
    Some(|_arguments: Vec<ParserNode>,| {     // 87
        let value;
        destruct_vec!(_arguments, _arg1);
        value = ParamList::make_list(_arg1.into());
        value
    }),
    Some(|_arguments: Vec<ParserNode>,| {     // 88
        let value;
        destruct_vec!(_arguments, _arg1, _arg2, _arg3);
        value = ParamList::push(_arg1.into(), _arg2.into(), _arg3.into());
        value
    }),
    Some(|_arguments: Vec<ParserNode>,| {     // 89
        let value;
        destruct_vec!(_arguments, _arg1, _arg2);
        value = ParamDecl::make(_arg1.into(), Some(_arg2.into()), true);
        value
    }),
    Some(|_arguments: Vec<ParserNode>,| {     // 90
        let value;
        destruct_vec!(_arguments, _arg1, _arg2);
        value = ParamDecl::make(_arg1.into(), _arg2.into(), false);
        value
    }),
    Some(|_arguments: Vec<ParserNode>,| {     // 91
        let value;
        value = ParserNode::None;
        value
    }),
    Some(|_arguments: Vec<ParserNode>,| {     // 92
        let value;
        destruct_vec!(_arguments, _arg1);
        value = _arg1.into();
        value
    }),
    Some(|_arguments: Vec<ParserNode>,| {     // 93
        let value;
        destruct_vec!(_arguments, _arg1);
        value = Declarator::make(Some(_arg1.into()), None);
        value
    }),
    Some(|_arguments: Vec<ParserNode>,| {     // 94
        let value;
        destruct_vec!(_arguments, _arg1, _arg2);
        value = Declarator::make(_arg1.into(), Some(_arg2.into()));
        value
    }),
    Some(|_arguments: Vec<ParserNode>,| {     // 95
        let value;
        destruct_vec!(_arguments, _arg1, _arg2, _arg3);
        value = DeclChunk::make_list( DeclChunk::make_paren(_arg1.into(), _arg2.into(), _arg3.into()) );
        value
    }),
    Some(|_arguments: Vec<ParserNode>,| {     // 96
        let value;
        destruct_vec!(_arguments, _arg1, _arg2, _arg3);
        value = DeclChunk::make_list( DeclChunk::make_array(_arg1.into(), _arg2.into(), _arg3.into()) );
        value
    }),
    Some(|_arguments: Vec<ParserNode>,| {     // 97
        let value;
        destruct_vec!(_arguments, _arg1, _arg2, _arg3, _arg4);
        value = DeclChunk::push( _arg1.into(), DeclChunk::make_array(_arg2.into(), _arg3.into(), _arg4.into()) );
        value
    }),
    Some(|_arguments: Vec<ParserNode>,| {     // 98
        let value;
        destruct_vec!(_arguments, _arg1, _arg2, _arg3);
        value = DeclChunk::make_list( DeclChunk::make_function(_arg1.into(), _arg2.into(), _arg3.into()) );
        value
    }),
    Some(|_arguments: Vec<ParserNode>,| {     // 99
        let value;
        destruct_vec!(_arguments, _arg1, _arg2, _arg3, _arg4);
        value = DeclChunk::push( _arg1.into(), DeclChunk::make_function(_arg2.into(), _arg3.into(), _arg4.into()) );
        value
    }),
    Some(|_arguments: Vec<ParserNode>,| {     // 100
        let value;
        value = ParserNode::None;
        value
    }),
    Some(|_arguments: Vec<ParserNode>,| {     // 101
        let value;
        destruct_vec!(_arguments, _arg1);
        value = _arg1.into();
        value
    }),
    Some(|_arguments: Vec<ParserNode>,| {     // 102
        let value;
        destruct_vec!(_arguments, _arg1);
        value = InitInfo::make_expr(_arg1.into());
        value
    }),
    Some(|_arguments: Vec<ParserNode>,| {     // 103
        let value;
        destruct_vec!(_arguments, _arg1, _arg2, _arg3);
        value = InitInfo::make_init_list(_arg1.into(), _arg2.into(), None, _arg3.into());
        value
    }),
    Some(|_arguments: Vec<ParserNode>,| {     // 104
        let value;
        destruct_vec!(_arguments, _arg1, _arg2, _arg3, _arg4);
        value = InitInfo::make_init_list(_arg1.into(), _arg2.into(), Some(_arg3.into()), _arg4.into());
        value
    }),
    Some(|_arguments: Vec<ParserNode>,| {     // 105
        let value;
        destruct_vec!(_arguments, _arg1);
        value = InitInfo::make_list(_arg1.into());
        value
    }),
    Some(|_arguments: Vec<ParserNode>,| {     // 106
        let value;
        destruct_vec!(_arguments, _arg1, _arg2, _arg3);
        value = InitInfo::push(_arg1.into(), _arg2.into(), _arg3.into());
        value
    }),
    Some(|_arguments: Vec<ParserNode>,| {     // 107
        let value;
        destruct_vec!(_arguments, _arg1);
        value = _arg1.into();
        value
    }),
    Some(|_arguments: Vec<ParserNode>,| {     // 108
        let value;
        destruct_vec!(_arguments, _arg1);
        value = _arg1.into();
        value
    }),
    Some(|_arguments: Vec<ParserNode>,| {     // 109
        let value;
        destruct_vec!(_arguments, _arg1);
        value = _arg1.into();
        value
    }),
    Some(|_arguments: Vec<ParserNode>,| {     // 110
        let value;
        destruct_vec!(_arguments, _arg1);
        value = _arg1.into();
        value
    }),
    Some(|_arguments: Vec<ParserNode>,| {     // 111
        let value;
        destruct_vec!(_arguments, _arg1);
        value = _arg1.into();
        value
    }),
    Some(|_arguments: Vec<ParserNode>,| {     // 112
        let value;
        destruct_vec!(_arguments, _arg1);
        value = _arg1.into();
        value
    }),
    Some(|_arguments: Vec<ParserNode>,| {     // 113
        let value;
        destruct_vec!(_arguments, _arg1, _arg2, _arg3);
        value = Statement::make_label(_arg1.into(), _arg3.into());
        value
    }),
    Some(|_arguments: Vec<ParserNode>,| {     // 114
        let value;
        destruct_vec!(_arguments, _arg1, _arg2, _arg3, _arg4);
        value = Statement::make_case(_arg1.into(), _arg2.into(), _arg4.into());
        value
    }),
    Some(|_arguments: Vec<ParserNode>,| {     // 115
        let value;
        destruct_vec!(_arguments, _arg1, _arg2, _arg3);
        value = Statement::make_default(_arg1.into(), _arg3.into());
        value
    }),
    Some(|_arguments: Vec<ParserNode>,| {     // 116
        let value;
        destruct_vec!(_arguments, _arg1, _arg2);
        value = Statement::make_compound(_arg1.into(), None, _arg2.into());
        value
    }),
    Some(|_arguments: Vec<ParserNode>,| {     // 117
        let value;
        destruct_vec!(_arguments, _arg1, _arg2, _arg3);
        value = Statement::make_compound(_arg1.into(), Some(_arg2.into()), _arg3.into());
        value
    }),
    Some(|_arguments: Vec<ParserNode>,| {     // 118
        let value;
        destruct_vec!(_arguments, _arg1);
        value = BlockItem::make_list(_arg1.into());
        value
    }),
    Some(|_arguments: Vec<ParserNode>,| {     // 119
        let value;
        destruct_vec!(_arguments, _arg1, _arg2);
        value = BlockItem::push(_arg1.into(), _arg2.into());
        value
    }),
    Some(|_arguments: Vec<ParserNode>,| {     // 120
        let value;
        destruct_vec!(_arguments, _arg1);
        value = BlockItem::make_decl(_arg1.into());
        value
    }),
    Some(|_arguments: Vec<ParserNode>,| {     // 121
        let value;
        destruct_vec!(_arguments, _arg1);
        value = BlockItem::make_stmt(_arg1.into());
        value
    }),
    Some(|_arguments: Vec<ParserNode>,| {     // 122
        let value;
        destruct_vec!(_arguments, _arg1);
        value = Statement::make_expression(None, _arg1.into());
        value
    }),
    Some(|_arguments: Vec<ParserNode>,| {     // 123
        let value;
        destruct_vec!(_arguments, _arg1, _arg2);
        value = Statement::make_expression(Some(_arg1.into()), _arg2.into());
        value
    }),
    Some(|_arguments: Vec<ParserNode>,| {     // 124
        let value;
        destruct_vec!(_arguments, _arg1, _arg2, _arg3, _arg4, _arg5);
        value = Statement::make_if(_arg1.into(), _arg3.into(), _arg5.into(), None);
        value
    }),
    Some(|_arguments: Vec<ParserNode>,| {     // 125
        let value;
        destruct_vec!(_arguments, _arg1, _arg2, _arg3, _arg4, _arg5, _arg6, _arg7);
        value = Statement::make_if(_arg1.into(), _arg3.into(), _arg5.into(), Some(_arg7.into()));
        value
    }),
    Some(|_arguments: Vec<ParserNode>,| {     // 126
        let value;
        destruct_vec!(_arguments, _arg1, _arg2, _arg3, _arg4, _arg5);
        value = Statement::make_switch(_arg1.into(), _arg3.into(), _arg5.into());
        value
    }),
    Some(|_arguments: Vec<ParserNode>,| {     // 127
        let value;
        destruct_vec!(_arguments, _arg1, _arg2, _arg3, _arg4, _arg5);
        value = Statement::make_while(_arg1.into(), _arg3.into(), _arg5.into(), None);
        value
    }),
    Some(|_arguments: Vec<ParserNode>,| {     // 128
        let value;
        destruct_vec!(_arguments, _arg1, _arg2, _arg3, _arg4, _arg5, _arg6, _arg7);
        value = Statement::make_while(_arg1.into(), _arg2.into(), _arg5.into(), Some(_arg6.into()));
        value
    }),
    Some(|_arguments: Vec<ParserNode>,| {     // 129
        let value;
        destruct_vec!(_arguments, _arg1, _arg2, _arg3, _arg4, _arg5, _arg6, _arg7, _arg8, _arg9);
        value = Statement::make_for(_arg1.into(), _arg3.into(), _arg5.into(), _arg7.into(), _arg9.into());
        value
    }),
    Some(|_arguments: Vec<ParserNode>,| {     // 130
        let value;
        value = ParserNode::None;
        value
    }),
    Some(|_arguments: Vec<ParserNode>,| {     // 131
        let value;
        destruct_vec!(_arguments, _arg1);
        value = _arg1.into();
        value
    }),
    Some(|_arguments: Vec<ParserNode>,| {     // 132
        let value;
        destruct_vec!(_arguments, _arg1, _arg2, _arg3);
        value = Statement::make_goto(_arg1.into(), _arg2.into());
        value
    }),
    Some(|_arguments: Vec<ParserNode>,| {     // 133
        let value;
        destruct_vec!(_arguments, _arg1, _arg2);
        value = Statement::make_continue_break(_arg1.into());
        value
    }),
    Some(|_arguments: Vec<ParserNode>,| {     // 134
        let value;
        destruct_vec!(_arguments, _arg1, _arg2);
        value = Statement::make_continue_break(_arg1.into());
        value
    }),
    Some(|_arguments: Vec<ParserNode>,| {     // 135
        let value;
        destruct_vec!(_arguments, _arg1, _arg2);
        value = Statement::make_return(_arg1.into(), None);
        value
    }),
    Some(|_arguments: Vec<ParserNode>,| {     // 136
        let value;
        destruct_vec!(_arguments, _arg1, _arg2, _arg3);
        value = Statement::make_return(_arg1.into(), _arg2.into());
        value
    }),
    Some(|_arguments: Vec<ParserNode>,| {     // 137
        let value;
        destruct_vec!(_arguments, _arg1);
        value = Expression::make_id(_arg1.into());
        value
    }),
    Some(|_arguments: Vec<ParserNode>,| {     // 138
        let value;
        destruct_vec!(_arguments, _arg1);
        value = Expression::make_literal(_arg1.into());
        value
    }),
    Some(|_arguments: Vec<ParserNode>,| {     // 139
        let value;
        destruct_vec!(_arguments, _arg1);
        value = Expression::make_literal(_arg1.into());
        value
    }),
    Some(|_arguments: Vec<ParserNode>,| {     // 140
        let value;
        destruct_vec!(_arguments, _arg1, _arg2, _arg3);
        value = _arg2.into();
        value
    }),
    Some(|_arguments: Vec<ParserNode>,| {     // 141
        let value;
        destruct_vec!(_arguments, _arg1);
        value = Constant::make(_arg1.into());
        value
    }),
    Some(|_arguments: Vec<ParserNode>,| {     // 142
        let value;
        destruct_vec!(_arguments, _arg1);
        value = Constant::make(_arg1.into());
        value
    }),
    Some(|_arguments: Vec<ParserNode>,| {     // 143
        let value;
        destruct_vec!(_arguments, _arg1);
        value = Constant::make(_arg1.into());
        value
    }),
    Some(|_arguments: Vec<ParserNode>,| {     // 144
        let value;
        destruct_vec!(_arguments, _arg1);
        value = Constant::make(_arg1.into());
        value
    }),
    Some(|_arguments: Vec<ParserNode>,| {     // 145
        let value;
        destruct_vec!(_arguments, _arg1, _arg2);
        value = Constant::insert_str(_arg1.into(), _arg2.into());
        value
    }),
    Some(|_arguments: Vec<ParserNode>,| {     // 146
        let value;
        destruct_vec!(_arguments, _arg1);
        value = _arg1.into();
        value
    }),
    Some(|_arguments: Vec<ParserNode>,| {     // 147
        let value;
        destruct_vec!(_arguments, _arg1, _arg2, _arg3, _arg4);
        value = Expression::make_array_access(_arg1.into(), _arg3.into(), _arg4.into());
        value
    }),
    Some(|_arguments: Vec<ParserNode>,| {     // 148
        let value;
        destruct_vec!(_arguments, _arg1, _arg2, _arg3, _arg4);
        value = Expression::make_call(_arg1.into(), _arg3.into(), _arg4.into());
        value
    }),
    Some(|_arguments: Vec<ParserNode>,| {     // 149
        let value;
        destruct_vec!(_arguments, _arg1, _arg2, _arg3);
        value = Expression::make_field_access(_arg1.into(), _arg3.into());
        value
    }),
    Some(|_arguments: Vec<ParserNode>,| {     // 150
        let value;
        destruct_vec!(_arguments, _arg1, _arg2, _arg3);
        value = Expression::make_arrow(_arg1.into(), _arg3.into());
        value
    }),
    Some(|_arguments: Vec<ParserNode>,| {     // 151
        let value;
        destruct_vec!(_arguments, _arg1, _arg2);
        value = Expression::make_update(_arg1.into(), _arg2.into(), true);
        value
    }),
    Some(|_arguments: Vec<ParserNode>,| {     // 152
        let value;
        destruct_vec!(_arguments, _arg1, _arg2);
        value = Expression::make_update(_arg1.into(), _arg2.into(), true);
        value
    }),
    Some(|_arguments: Vec<ParserNode>,| {     // 153
        let value;
        value = ParserNode::None;
        value
    }),
    Some(|_arguments: Vec<ParserNode>,| {     // 154
        let value;
        destruct_vec!(_arguments, _arg1);
        value = _arg1.into();
        value
    }),
    Some(|_arguments: Vec<ParserNode>,| {     // 155
        let value;
        destruct_vec!(_arguments, _arg1);
        value = _arg1.into();
        value
    }),
    Some(|_arguments: Vec<ParserNode>,| {     // 156
        let value;
        destruct_vec!(_arguments, _arg1, _arg2, _arg3);
        value = Expression::make_assign(_arg1.into(), _arg2.into(), _arg3.into());
        value
    }),
    Some(|_arguments: Vec<ParserNode>,| {     // 157
        let value;
        destruct_vec!(_arguments, _arg1);
        value = _arg1.into();
        value
    }),
    Some(|_arguments: Vec<ParserNode>,| {     // 158
        let value;
        destruct_vec!(_arguments, _arg1, _arg2);
        value = Expression::make_update(_arg2.into(), _arg1.into(), false);
        value
    }),
    Some(|_arguments: Vec<ParserNode>,| {     // 159
        let value;
        destruct_vec!(_arguments, _arg1, _arg2);
        value = Expression::make_update(_arg2.into(), _arg1.into(), true);
        value
    }),
    Some(|_arguments: Vec<ParserNode>,| {     // 160
        let value;
        destruct_vec!(_arguments, _arg1, _arg2);
        value = Expression::make_unary(_arg1.into(), _arg2.into());
        value
    }),
    Some(|_arguments: Vec<ParserNode>,| {     // 161
        let value;
        destruct_vec!(_arguments, _arg1, _arg2);
        value = Expression::make_sizeof_expr(_arg1.into(), _arg2.into());
        value
    }),
    Some(|_arguments: Vec<ParserNode>,| {     // 162
        let value;
        destruct_vec!(_arguments, _arg1, _arg2, _arg3, _arg4);
        value = Expression::make_sizeof_type(_arg1.into(), _arg2.into(), _arg3.into(), _arg4.into());
        value
    }),
    Some(|_arguments: Vec<ParserNode>,| {     // 163
        let value;
        destruct_vec!(_arguments, _arg1);
        value = _arg1.into();
        value
    }),
    Some(|_arguments: Vec<ParserNode>,| {     // 164
        let value;
        destruct_vec!(_arguments, _arg1);
        value = _arg1.into();
        value
    }),
    Some(|_arguments: Vec<ParserNode>,| {     // 165
        let value;
        destruct_vec!(_arguments, _arg1);
        value = _arg1.into();
        value
    }),
    Some(|_arguments: Vec<ParserNode>,| {     // 166
        let value;
        destruct_vec!(_arguments, _arg1);
        value = _arg1.into();
        value
    }),
    Some(|_arguments: Vec<ParserNode>,| {     // 167
        let value;
        destruct_vec!(_arguments, _arg1);
        value = _arg1.into();
        value
    }),
    Some(|_arguments: Vec<ParserNode>,| {     // 168
        let value;
        destruct_vec!(_arguments, _arg1);
        value = _arg1.into();
        value
    }),
    Some(|_arguments: Vec<ParserNode>,| {     // 169
        let value;
        destruct_vec!(_arguments, _arg1, _arg2, _arg3, _arg4);
        value = Expression::make_cast(_arg1.into(), _arg2.into(), _arg4.into());
        value
    }),
    Some(|_arguments: Vec<ParserNode>,| {     // 170
        let value;
        destruct_vec!(_arguments, _arg1);
        value = _arg1.into();
        value
    }),
    Some(|_arguments: Vec<ParserNode>,| {     // 171
        let value;
        destruct_vec!(_arguments, _arg1, _arg2, _arg3);
        value = Expression::make_binary(_arg1.into(), _arg2.into(), _arg3.into());
        value
    }),
    Some(|_arguments: Vec<ParserNode>,| {     // 172
        let value;
        destruct_vec!(_arguments, _arg1, _arg2, _arg3);
        value = Expression::make_binary(_arg1.into(), _arg2.into(), _arg3.into());
        value
    }),
    Some(|_arguments: Vec<ParserNode>,| {     // 173
        let value;
        destruct_vec!(_arguments, _arg1, _arg2, _arg3);
        value = Expression::make_binary(_arg1.into(), _arg2.into(), _arg3.into());
        value
    }),
    Some(|_arguments: Vec<ParserNode>,| {     // 174
        let value;
        destruct_vec!(_arguments, _arg1);
        value = _arg1.into();
        value
    }),
    Some(|_arguments: Vec<ParserNode>,| {     // 175
        let value;
        destruct_vec!(_arguments, _arg1, _arg2, _arg3);
        value = Expression::make_binary(_arg1.into(), _arg2.into(), _arg3.into());
        value
    }),
    Some(|_arguments: Vec<ParserNode>,| {     // 176
        let value;
        destruct_vec!(_arguments, _arg1, _arg2, _arg3);
        value = Expression::make_binary(_arg1.into(), _arg2.into(), _arg3.into());
        value
    }),
    Some(|_arguments: Vec<ParserNode>,| {     // 177
        let value;
        destruct_vec!(_arguments, _arg1);
        value = _arg1.into();
        value
    }),
    Some(|_arguments: Vec<ParserNode>,| {     // 178
        let value;
        destruct_vec!(_arguments, _arg1, _arg2, _arg3);
        value = Expression::make_binary(_arg1.into(), _arg2.into(), _arg3.into());
        value
    }),
    Some(|_arguments: Vec<ParserNode>,| {     // 179
        let value;
        destruct_vec!(_arguments, _arg1, _arg2, _arg3);
        value = Expression::make_binary(_arg1.into(), _arg2.into(), _arg3.into());
        value
    }),
    Some(|_arguments: Vec<ParserNode>,| {     // 180
        let value;
        destruct_vec!(_arguments, _arg1);
        value = _arg1.into();
        value
    }),
    Some(|_arguments: Vec<ParserNode>,| {     // 181
        let value;
        destruct_vec!(_arguments, _arg1, _arg2, _arg3);
        value = Expression::make_binary(_arg1.into(), _arg2.into(), _arg3.into());
        value
    }),
    Some(|_arguments: Vec<ParserNode>,| {     // 182
        let value;
        destruct_vec!(_arguments, _arg1, _arg2, _arg3);
        value = Expression::make_binary(_arg1.into(), _arg2.into(), _arg3.into());
        value
    }),
    Some(|_arguments: Vec<ParserNode>,| {     // 183
        let value;
        destruct_vec!(_arguments, _arg1, _arg2, _arg3);
        value = Expression::make_binary(_arg1.into(), _arg2.into(), _arg3.into());
        value
    }),
    Some(|_arguments: Vec<ParserNode>,| {     // 184
        let value;
        destruct_vec!(_arguments, _arg1, _arg2, _arg3);
        value = Expression::make_binary(_arg1.into(), _arg2.into(), _arg3.into());
        value
    }),
    Some(|_arguments: Vec<ParserNode>,| {     // 185
        let value;
        destruct_vec!(_arguments, _arg1);
        value = _arg1.into();
        value
    }),
    Some(|_arguments: Vec<ParserNode>,| {     // 186
        let value;
        destruct_vec!(_arguments, _arg1, _arg2, _arg3);
        value = Expression::make_binary(_arg1.into(), _arg2.into(), _arg3.into());
        value
    }),
    Some(|_arguments: Vec<ParserNode>,| {     // 187
        let value;
        destruct_vec!(_arguments, _arg1, _arg2, _arg3);
        value = Expression::make_binary(_arg1.into(), _arg2.into(), _arg3.into());
        value
    }),
    Some(|_arguments: Vec<ParserNode>,| {     // 188
        let value;
        destruct_vec!(_arguments, _arg1);
        value = _arg1.into();
        value
    }),
    Some(|_arguments: Vec<ParserNode>,| {     // 189
        let value;
        destruct_vec!(_arguments, _arg1, _arg2, _arg3);
        value = Expression::make_binary(_arg1.into(), _arg2.into(), _arg3.into());
        value
    }),
    Some(|_arguments: Vec<ParserNode>,| {     // 190
        let value;
        destruct_vec!(_arguments, _arg1);
        value = _arg1.into();
        value
    }),
    Some(|_arguments: Vec<ParserNode>,| {     // 191
        let value;
        destruct_vec!(_arguments, _arg1, _arg2, _arg3);
        value = Expression::make_binary(_arg1.into(), _arg2.into(), _arg3.into());
        value
    }),
    Some(|_arguments: Vec<ParserNode>,| {     // 192
        let value;
        destruct_vec!(_arguments, _arg1);
        value = _arg1.into();
        value
    }),
    Some(|_arguments: Vec<ParserNode>,| {     // 193
        let value;
        destruct_vec!(_arguments, _arg1, _arg2, _arg3);
        value = Expression::make_binary(_arg1.into(), _arg2.into(), _arg3.into());
        value
    }),
    Some(|_arguments: Vec<ParserNode>,| {     // 194
        let value;
        destruct_vec!(_arguments, _arg1);
        value = _arg1.into();
        value
    }),
    Some(|_arguments: Vec<ParserNode>,| {     // 195
        let value;
        destruct_vec!(_arguments, _arg1, _arg2, _arg3);
        value = Expression::make_binary(_arg1.into(), _arg2.into(), _arg3.into());
        value
    }),
    Some(|_arguments: Vec<ParserNode>,| {     // 196
        let value;
        destruct_vec!(_arguments, _arg1);
        value = _arg1.into();
        value
    }),
    Some(|_arguments: Vec<ParserNode>,| {     // 197
        let value;
        destruct_vec!(_arguments, _arg1, _arg2, _arg3);
        value = Expression::make_binary(_arg1.into(), _arg2.into(), _arg3.into());
        value
    }),
    Some(|_arguments: Vec<ParserNode>,| {     // 198
        let value;
        destruct_vec!(_arguments, _arg1);
        value = _arg1.into();
        value
    }),
    Some(|_arguments: Vec<ParserNode>,| {     // 199
        let value;
        destruct_vec!(_arguments, _arg1);
        value = _arg1.into();
        value
    }),
    Some(|_arguments: Vec<ParserNode>,| {     // 200
        let value;
        destruct_vec!(_arguments, _arg1, _arg2, _arg3, _arg4, _arg5);
        value = Expression::make_conditional(_arg1.into(), _arg3.into(), _arg5.into());
        value
    }),
    Some(|_arguments: Vec<ParserNode>,| {     // 201
        let value;
        destruct_vec!(_arguments, _arg1);
        value = _arg1.into();
        value
    }),
    Some(|_arguments: Vec<ParserNode>,| {     // 202
        let value;
        destruct_vec!(_arguments, _arg1, _arg2, _arg3);
        value = Expression::make_assign(_arg1.into(), _arg2.into(), _arg3.into());
        value
    }),
    Some(|_arguments: Vec<ParserNode>,| {     // 203
        let value;
        destruct_vec!(_arguments, _arg1);
        value = _arg1.into();
        value
    }),
    Some(|_arguments: Vec<ParserNode>,| {     // 204
        let value;
        destruct_vec!(_arguments, _arg1);
        value = _arg1.into();
        value
    }),
    Some(|_arguments: Vec<ParserNode>,| {     // 205
        let value;
        destruct_vec!(_arguments, _arg1);
        value = _arg1.into();
        value
    }),
    Some(|_arguments: Vec<ParserNode>,| {     // 206
        let value;
        destruct_vec!(_arguments, _arg1);
        value = _arg1.into();
        value
    }),
    Some(|_arguments: Vec<ParserNode>,| {     // 207
        let value;
        destruct_vec!(_arguments, _arg1);
        value = _arg1.into();
        value
    }),
    Some(|_arguments: Vec<ParserNode>,| {     // 208
        let value;
        destruct_vec!(_arguments, _arg1);
        value = _arg1.into();
        value
    }),
    Some(|_arguments: Vec<ParserNode>,| {     // 209
        let value;
        destruct_vec!(_arguments, _arg1);
        value = _arg1.into();
        value
    }),
    Some(|_arguments: Vec<ParserNode>,| {     // 210
        let value;
        destruct_vec!(_arguments, _arg1);
        value = _arg1.into();
        value
    }),
    Some(|_arguments: Vec<ParserNode>,| {     // 211
        let value;
        destruct_vec!(_arguments, _arg1);
        value = _arg1.into();
        value
    }),
    Some(|_arguments: Vec<ParserNode>,| {     // 212
        let value;
        destruct_vec!(_arguments, _arg1);
        value = _arg1.into();
        value
    }),
    Some(|_arguments: Vec<ParserNode>,| {     // 213
        let value;
        destruct_vec!(_arguments, _arg1);
        value = _arg1.into();
        value
    }),
    Some(|_arguments: Vec<ParserNode>,| {     // 214
        let value;
        destruct_vec!(_arguments, _arg1);
        value = _arg1.into();
        value
    }),
    Some(|_arguments: Vec<ParserNode>,| {     // 215
        let value;
        destruct_vec!(_arguments, _arg1, _arg2, _arg3);
        value = Expression::make_comma(_arg1.into(), _arg3.into());
        value
    }),
    Some(|_arguments: Vec<ParserNode>,| {     // 216
        let value;
        destruct_vec!(_arguments, _arg1);
        value = _arg1.into();
        value
    }),
    Some(|_arguments: Vec<ParserNode>,| {     // 217
        let value;
        destruct_vec!(_arguments, _arg1, _arg2);
        value = CompleteDecl::make(_arg1.into(), _arg2.into());
        value
    }),
];



/// action_table[state][token]
pub fn get_action(state: usize, token: usize) -> LRAction {
    let row_id = ACTION_ROW_ID[state];
    let base = ACTION_BASE[row_id];
    if base.is_none() {
        return Error
    }

    let idx = base.unwrap() + token;

    let check = match ACTION_CHECK[idx] {
        None => return Error,
        Some(x) => x
    };

    if check == row_id {
        ACTION_NEXT[idx].clone()
    } else {
        Error
    }
}

/// action_table[state][EXPR_IDS[prod_id]]
pub fn get_goto(state: usize, prod_id: usize) -> Option<usize> {
    let row_id = GOTO_ROW_ID[state];
    let rule_id = EXPR_IDS[prod_id];
    let base = GOTO_BASE[row_id]?;

    let idx = base + rule_id;

    let check = GOTO_CHECK[idx]?;

    if check == row_id {
        GOTO_NEXT[idx]
    } else {
        None
    }
}


