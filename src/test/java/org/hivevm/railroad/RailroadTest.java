package org.hivevm.railroad;

import org.hivevm.railroad.bnf.BNFParser;
import org.hivevm.railroad.bnf.BNFWriter;
import org.hivevm.railroad.diagram.Railroad;
import org.hivevm.railroad.grammar.*;
import org.hivevm.railroad.svg.SvgDiagram;
import org.hivevm.railroad.svg.SvgLayout;
import org.junit.jupiter.api.Test;
import org.w3c.dom.Document;
import org.xml.sax.InputSource;

import javax.xml.parsers.DocumentBuilderFactory;
import java.io.IOException;
import java.io.StringReader;
import java.util.Arrays;
import java.util.List;

import static org.junit.jupiter.api.Assertions.assertEquals;

public class RailroadTest {

    @Test
    public void testConsecutiveRuleReferencesSeparatedByNewline() {
        assertEquals(5, countElements("rect", svg("rule = a b\nc d\n\te;", new SvgLayout())));
        assertEquals(5, countElements("rect", svg("rule = a b\r\nc d\ne;", new SvgLayout())));
    }

    @Test
    public void testGrammarToString() {
        assertEquals("r = a b;", BNFWriter.toBNF(grammar("r = a b;")));
        assertEquals("r1 = a b;\nr2 = c d;", BNFWriter.toBNF(grammar("r1 = a b;\nr2 = c d;")));
    }

    @Test
    public void testRuleToString() {
        assertEquals("r = a b;", BNFWriter.toBNF(rule("r = a b;")));
    }

    @Test
    public void testChoiceToString() {
        List<Expression> list = Arrays.asList(new RuleReference("a"), new RuleReference("b"));
        assertEquals("a | b", BNFWriter.toBNF(new Choice(list)));
    }

    @Test
    public void testLiteralToString() {
        assertEquals("'a'", BNFWriter.toBNF(new Literal("a")));
    }

    @Test
    public void testRuleReferenceToString() {
        assertEquals("a", BNFWriter.toBNF(new RuleReference("a")));
    }

    @Test
    public void testSequenceToString() {
        List<Expression> list = Arrays.asList(new RuleReference("a"), new RuleReference("b"));
        assertEquals("a b", BNFWriter.toBNF(new Sequence(list)));
    }

    @Test
    public void testSpecialSequenceToString() {
        assertEquals("(? abc ?)", BNFWriter.toBNF(new SpecialSequence("abc")));
    }

    @Test
    public void testRepetitionToString() {
        assertEquals("{ a }", BNFWriter.toBNF(new Repetition(new RuleReference("a"), 0, null)));
        assertEquals("[ a ]", BNFWriter.toBNF(new Repetition(new RuleReference("a"), 0, 1)));
        assertEquals("2 * [ a ]", BNFWriter.toBNF(new Repetition(new RuleReference("a"), 0, 2)));
    }

    @Test
    public void testConversionsToBNF() {
        String bnf1 =
                "BNF1 = a*;" +
                        "BNF2 = a+;" +
                        "BNF3 = a?;" +
                        "BNF4 = a (',' a)*;" +
                        "BNF5 = 3 * a;" +
                        "BNF6 = 3 * a?;" +
                        "BNF7 = a 3 * (',' a);" +
                        "BNF8 = a 3 * (',' a)?;" +
                        "BNF9 = a (? [a-zA-Z]+ ?) 3 * (',' a)?;" +
                        "BNF10 = a | c | ();" +
                        "BNF11 = 3 * 'a<b\"';";
        Grammar grammar1 = grammar(bnf1);
        String bnf2 = BNFWriter.toBNF(grammar1);
        // Resulting BNF may be different depending on format options.
        // Nevertheless, they should be equivalent, and one way to test this is
        // to produce the both SVG and compare them.
//    System.err.println(bnf2);
        Grammar grammar2 = grammar(bnf2);
        var rules1 = grammar1.rules();
        var rules2 = grammar2.rules();
        assertEquals(rules1.size(), rules2.size());
        for (int i = 0; i < rules1.size(); i++) {
            Rule rule1 = rules1.get(i);
            Rule rule2 = rules2.get(i);
            assertEquals(rule1.name(), rule2.name(), "Rules have same name");
            SvgDiagram diagram1 = SvgDiagram.toDiagram(Railroad.toRailroad(rule1.expression()));
            String svg1 = diagram1.toSVG(new SvgLayout());
            SvgDiagram diagram2 = SvgDiagram.toDiagram(Railroad.toRailroad(rule2.expression()));
            String svg2 = diagram2.toSVG(new SvgLayout());
//      System.err.println("SVG1: " + svg1);
//      System.err.println("SVG2: " + svg2);
            assertEquals(svg1, svg2, "SVG for \"" + rule1.name() + "\" are identical");
        }
    }

    @Test
    public void testEndShape() {
        final String grammar = "rule = a b;";
        String svg;

        // plain ends (default)
        svg = svg(grammar, new SvgLayout());
        assertEquals(2, countElements("rect", svg));
        assertEquals(0, countElements("ellipse", svg));
        assertEquals(0, countElements("line", svg));
        //saveFile(svg, "plain.svg");

        // circle ends
        var renderer = new SvgLayout();
        renderer.setStartElement(new Railroad.ShapeElement(Railroad.StartEndShape.EMPTY_CIRCLE, true));
        renderer.setEndElement(new Railroad.ShapeElement(Railroad.StartEndShape.EMPTY_CIRCLE, false));
        svg = svg(grammar, renderer);
        assertEquals(2, countElements("rect", svg));
        assertEquals(2, countElements("ellipse", svg));
        assertEquals(0, countElements("line", svg));
        //saveFile(svg, "circle.svg");

        // single cross ends
        renderer = new SvgLayout();
        renderer.setStartElement(new Railroad.ShapeElement(Railroad.StartEndShape.VERTICAL_LINE, true));
        renderer.setEndElement(new Railroad.ShapeElement(Railroad.StartEndShape.VERTICAL_LINE, false));
        svg = svg(grammar, renderer);
        assertEquals(2, countElements("rect", svg));
        assertEquals(0, countElements("ellipse", svg));
        assertEquals(2, countElements("line", svg));
        //saveFile(svg, "cross.svg");

        // double cross ends
        renderer = new SvgLayout();
        renderer.setStartElement(new Railroad.ShapeElement(Railroad.StartEndShape.DOUBLE_VERTICAL_LINE, true));
        renderer.setEndElement(new Railroad.ShapeElement(Railroad.StartEndShape.DOUBLE_VERTICAL_LINE, false));
        svg = svg(grammar, renderer);
        assertEquals(2, countElements("rect", svg));
        assertEquals(0, countElements("ellipse", svg));
        assertEquals(4, countElements("line", svg));
        //saveFile(svg, "double-cross.svg");
    }

    // Test utilities

    private String svg(String string, SvgLayout renderer) {
        var rule = grammar(string).rules().getFirst();
        var diagram = SvgDiagram.toDiagram(Railroad.toRailroad(rule.expression()));
        return diagram.toSVG(renderer);
    }

    private Grammar grammar(String string) {
        return BNFParser.parse(string);
    }

    private Rule rule(String string) {
        return grammar(string).rules().getFirst();
    }

    private int countElements(String tagName, String svg) {
        try {
            Document document = DocumentBuilderFactory.newInstance().newDocumentBuilder().parse(new InputSource(new StringReader(svg)));
            return document.getElementsByTagName(tagName).getLength();
        } catch (Exception e) {
            throw new RuntimeException(e);
        }
    }


    private static final String BNF =
            """
                    H2_SELECT =
                    'SELECT' [ 'TOP' term ] [ 'DISTINCT' | 'ALL' ] selectExpression {',' selectExpression} \
                    'FROM' tableExpression {',' tableExpression} [ 'WHERE' expression ] \
                    [ 'GROUP BY' expression {',' expression} ] [ 'HAVING' expression ] \
                    [ ( 'UNION' [ 'ALL' ] | 'MINUS' | 'EXCEPT' | 'INTERSECT' ) select ] [ 'ORDER BY' order {',' order} ] \
                    [ 'LIMIT' expression [ 'OFFSET' expression ] [ 'SAMPLE_SIZE' rowCountInt ] ] \
                    [ 'FOR UPDATE' ];
                    """;

    private static final String SEM_VER =
            """
                    sem_ver = major '.' minor [ '.' patch ]
                    [ '-' pre_release ] [ '+' build ];
                    <valid semver> ::= <version core> [ "-" <pre-release> ] [ "+" <build> ];
                    
                    <version core> ::= <major> "." <minor> "." <patch>;
                    
                    <major> ::= <numeric identifier>;
                    
                    <minor> ::= <numeric identifier>;
                    
                    <patch> ::= <numeric identifier>;
                    
                    <pre-release> ::= <dot-separated pre-release identifiers>;
                    
                    <dot-separated pre-release identifiers> ::= <pre-release identifier>
                                                              | <pre-release identifier> "." <dot-separated pre-release identifiers>;
                    
                    <build> ::= <dot-separated build identifiers>;
                    
                    <dot-separated build identifiers> ::= <build identifier>
                                                        | <build identifier> "." <dot-separated build identifiers>;
                    
                    <pre-release identifier> ::= <alphanumeric identifier>
                                               | <numeric identifier>;
                    
                    <build identifier> ::= <alphanumeric identifier>
                                         | <digits>;
                    
                    <alphanumeric identifier> ::= <non-digit>
                                                | <non-digit> <identifier characters>
                                                | <identifier characters> <non-digit>
                                                | <identifier characters> <non-digit> <identifier characters>;
                    
                    <numeric identifier> ::= "0"
                                           | <positive digit>
                                           | <positive digit> <digits>;
                    
                    <identifier characters> ::= <identifier character>
                                              | <identifier character> <identifier characters>;
                    
                    <identifier character> ::= <digit>
                                             | <non-digit>;
                    
                    <non-digit> ::= <letter>
                                  | "-";
                    
                    <digits> ::= <digit>
                               | <digit> <digits>;
                    <digit> ::= "0"
                              | <positive digit>;
                    <positive digit> ::= "1" | "2" | "3" | "4" | "5" | "6" | "7" | "8" | "9";
                    <letter> ::= "A" | "B" | "C" | "D" | "E" | "F" | "G" | "H" | "I" | "J"
                               | "K" | "L" | "M" | "N" | "O" | "P" | "Q" | "R" | "S" | "T"
                               | "U" | "V" | "W" | "X" | "Y" | "Z" | "a" | "b" | "c" | "d"
                               | "e" | "f" | "g" | "h" | "i" | "j" | "k" | "l" | "m" | "n"
                               | "o" | "p" | "q" | "r" | "s" | "t" | "u" | "v" | "w" | "x"
                               | "y" | "z";
                    """;

    private static final String SVG  = "<svg version=\"1.1\" xmlns:xlink=\"http://www.w3.org/1999/xlink\" xmlns=\"http://www.w3.org/2000/svg\" width=\"2716\" height=\"190\" viewbox=\"0 0 2716 190\"><defs><style type=\"text/css\">.c{fill:none;stroke:#222222;}.j{fill:#000000;font-family:Verdana,Sans-serif;font-size:12px;}.l{fill:#90d9ff;stroke:#222222;}.r{fill:#d3f0ff;stroke:#222222;}</style></defs><path class=\"c\" d=\"M0 53h5m62 0h30m43 0h10m47 0h20m-135 0q5 0 5 5v9q0 5 5 5h110q5 0 5-5v-9q0-5 5-5m5 0h30m75 0h20m-105 26q0 5 5 5h5m40 0h40q5 0 5-5m-100-26q5 0 5 5v35q0 5 5 5h85q5 0 5-5v-35q0-5 5-5m5 0h30m-5 0q-5 0-5-5v-21q0-5 5-5h50m23 0h51q5 0 5 5v21q0 5-5 5m-5 0h30m53 0h30m-5 0q-5 0-5-5v-21q0-5 5-5h48m23 0h48q5 0 5 5v21q0 5-5 5m-5 0h50m61 0h10m81 0h20m-187 0q5 0 5 5v9q0 5 5 5h162q5 0 5-5v-9q0-5 5-5m5 0h30m79 0h30m-5 0q-5 0-5-5v-21q0-5 5-5h34m23 0h34q5 0 5 5v21q0 5-5 5m-5 0h40m-245 0q5 0 5 5v9q0 5 5 5h220q5 0 5-5v-9q0-5 5-5m5 0h30m66 0h10m81 0h20m-192 0q5 0 5 5v9q0 5 5 5h167q5 0 5-5v-9q0-5 5-5m5 0h50m60 0h30m40 0h20m-75 0q5 0 5 5v9q0 5 5 5h50q5 0 5-5v-9q0-5 5-5m5 0h20m-180 41q0 5 5 5h5m60 0h95q5 0 5-5m-170 31q0 5 5 5h5m63 0h92q5 0 5-5m-175-72q5 0 5 5v98q0 5 5 5h5m83 0h72q5 0 5-5v-98q0-5 5-5m5 0h10m53 0h20m-288 0q5 0 5 5v117q0 5 5 5h263q5 0 5-5v-117q0-5 5-5m5 0h30m77 0h30m-5 0q-5 0-5-5v-21q0-5 5-5h19m23 0h19q5 0 5 5v21q0 5-5 5m-5 0h40m-213 0q5 0 5 5v9q0 5 5 5h188q5 0 5-5v-9q0-5 5-5m5 0h30m52 0h10m81 0h30m62 0h10m81 0h20m-188 0q5 0 5 5v9q0 5 5 5h163q5 0 5-5v-9q0-5 5-5m5 0h30m96 0h10m89 0h20m-230 0q5 0 5 5v9q0 5 5 5h205q5 0 5-5v-9q0-5 5-5m5 0h20m-626 0q5 0 5 5v24q0 5 5 5h601q5 0 5-5v-24q0-5 5-5m5 0h30m92 0h20m-127 0q5 0 5 5v9q0 5 5 5h102q5 0 5-5v-9q0-5 5-5m5 0h5\"/><rect class=\"l\" x=\"5\" y=\"36\" width=\"62\" height=\"26\" rx=\"7\"/><text class=\"j\" x=\"15\" y=\"53\">SELECT</text><rect class=\"l\" x=\"97\" y=\"36\" width=\"43\" height=\"26\" rx=\"7\"/><text class=\"j\" x=\"107\" y=\"53\">TOP</text><a xlink:href=\"#term\"><rect class=\"r\" x=\"150\" y=\"36\" width=\"47\" height=\"26\"/><text class=\"j\" x=\"160\" y=\"53\">term</text></a><rect class=\"l\" x=\"247\" y=\"36\" width=\"75\" height=\"26\" rx=\"7\"/><text class=\"j\" x=\"257\" y=\"53\">DISTINCT</text><rect class=\"l\" x=\"247\" y=\"67\" width=\"40\" height=\"26\" rx=\"7\"/><text class=\"j\" x=\"257\" y=\"84\">ALL</text><rect class=\"l\" x=\"417\" y=\"5\" width=\"23\" height=\"26\" rx=\"7\"/><text class=\"j\" x=\"427\" y=\"22\">,</text><a xlink:href=\"#selectExpression\"><rect class=\"r\" x=\"372\" y=\"36\" width=\"114\" height=\"26\"/><text class=\"j\" x=\"382\" y=\"53\">selectExpression</text></a><rect class=\"l\" x=\"516\" y=\"36\" width=\"53\" height=\"26\" rx=\"7\"/><text class=\"j\" x=\"526\" y=\"53\">FROM</text><rect class=\"l\" x=\"642\" y=\"5\" width=\"23\" height=\"26\" rx=\"7\"/><text class=\"j\" x=\"652\" y=\"22\">,</text><a xlink:href=\"#tableExpression\"><rect class=\"r\" x=\"599\" y=\"36\" width=\"109\" height=\"26\"/><text class=\"j\" x=\"609\" y=\"53\">tableExpression</text></a><rect class=\"l\" x=\"758\" y=\"36\" width=\"61\" height=\"26\" rx=\"7\"/><text class=\"j\" x=\"768\" y=\"53\">WHERE</text><a xlink:href=\"#expression\"><rect class=\"r\" x=\"829\" y=\"36\" width=\"81\" height=\"26\"/><text class=\"j\" x=\"839\" y=\"53\">expression</text></a><rect class=\"l\" x=\"960\" y=\"36\" width=\"79\" height=\"26\" rx=\"7\"/><text class=\"j\" x=\"970\" y=\"53\">GROUP BY</text><rect class=\"l\" x=\"1098\" y=\"5\" width=\"23\" height=\"26\" rx=\"7\"/><text class=\"j\" x=\"1108\" y=\"22\">,</text><a xlink:href=\"#expression\"><rect class=\"r\" x=\"1069\" y=\"36\" width=\"81\" height=\"26\"/><text class=\"j\" x=\"1079\" y=\"53\">expression</text></a><rect class=\"l\" x=\"1220\" y=\"36\" width=\"66\" height=\"26\" rx=\"7\"/><text class=\"j\" x=\"1230\" y=\"53\">HAVING</text><a xlink:href=\"#expression\"><rect class=\"r\" x=\"1296\" y=\"36\" width=\"81\" height=\"26\"/><text class=\"j\" x=\"1306\" y=\"53\">expression</text></a><rect class=\"l\" x=\"1447\" y=\"36\" width=\"60\" height=\"26\" rx=\"7\"/><text class=\"j\" x=\"1457\" y=\"53\">UNION</text><rect class=\"l\" x=\"1537\" y=\"36\" width=\"40\" height=\"26\" rx=\"7\"/><text class=\"j\" x=\"1547\" y=\"53\">ALL</text><rect class=\"l\" x=\"1447\" y=\"82\" width=\"60\" height=\"26\" rx=\"7\"/><text class=\"j\" x=\"1457\" y=\"99\">MINUS</text><rect class=\"l\" x=\"1447\" y=\"113\" width=\"63\" height=\"26\" rx=\"7\"/><text class=\"j\" x=\"1457\" y=\"130\">EXCEPT</text><rect class=\"l\" x=\"1447\" y=\"144\" width=\"83\" height=\"26\" rx=\"7\"/><text class=\"j\" x=\"1457\" y=\"161\">INTERSECT</text><a xlink:href=\"#select\"><rect class=\"r\" x=\"1627\" y=\"36\" width=\"53\" height=\"26\"/><text class=\"j\" x=\"1637\" y=\"53\">select</text></a><rect class=\"l\" x=\"1730\" y=\"36\" width=\"77\" height=\"26\" rx=\"7\"/><text class=\"j\" x=\"1740\" y=\"53\">ORDER BY</text><rect class=\"l\" x=\"1851\" y=\"5\" width=\"23\" height=\"26\" rx=\"7\"/><text class=\"j\" x=\"1861\" y=\"22\">,</text><a xlink:href=\"#order\"><rect class=\"r\" x=\"1837\" y=\"36\" width=\"51\" height=\"26\"/><text class=\"j\" x=\"1847\" y=\"53\">order</text></a><rect class=\"l\" x=\"1958\" y=\"36\" width=\"52\" height=\"26\" rx=\"7\"/><text class=\"j\" x=\"1968\" y=\"53\">LIMIT</text><a xlink:href=\"#expression\"><rect class=\"r\" x=\"2020\" y=\"36\" width=\"81\" height=\"26\"/><text class=\"j\" x=\"2030\" y=\"53\">expression</text></a><rect class=\"l\" x=\"2131\" y=\"36\" width=\"62\" height=\"26\" rx=\"7\"/><text class=\"j\" x=\"2141\" y=\"53\">OFFSET</text><a xlink:href=\"#expression\"><rect class=\"r\" x=\"2203\" y=\"36\" width=\"81\" height=\"26\"/><text class=\"j\" x=\"2213\" y=\"53\">expression</text></a><rect class=\"l\" x=\"2334\" y=\"36\" width=\"96\" height=\"26\" rx=\"7\"/><text class=\"j\" x=\"2344\" y=\"53\">SAMPLE_SIZE</text><a xlink:href=\"#rowCountInt\"><rect class=\"r\" x=\"2440\" y=\"36\" width=\"89\" height=\"26\"/><text class=\"j\" x=\"2450\" y=\"53\">rowCountInt</text></a><rect class=\"l\" x=\"2599\" y=\"36\" width=\"92\" height=\"26\" rx=\"7\"/><text class=\"j\" x=\"2609\" y=\"53\">FOR UPDATE</text></svg>";
    private static final String BNF1 = "H2_SELECT = 'SELECT' [ 'TOP' term ] [ 'DISTINCT' | 'ALL' ] selectExpression { ',' selectExpression } 'FROM' tableExpression { ',' tableExpression } [ 'WHERE' expression ] [ 'GROUP BY' expression { ',' expression } ] [ 'HAVING' expression ] [ ( 'UNION' [ 'ALL' ] | 'MINUS' | 'EXCEPT' | 'INTERSECT' ) select ] [ 'ORDER BY' order { ',' order } ] [ 'LIMIT' expression [ 'OFFSET' expression ] [ 'SAMPLE_SIZE' rowCountInt ] ] [ 'FOR UPDATE' ];";


    @Test
    public void testSvg() {
        var grammar = RailroadHandler.TO_GRAMMAR.handleRequest(BNF, null);

        assertEquals(SVG, RailroadHandler.TO_SVG.handleRequest(grammar, null));
        assertEquals(BNF1, RailroadHandler.TO_BNF.handleRequest(grammar, null));
    }

    public static void main(String[] args) throws IOException {
        var grammar = RailroadHandler.TO_GRAMMAR.handleRequest(BNF, null);

        System.out.println(RailroadHandler.TO_SVG.handleRequest(grammar, null));
        System.out.println(RailroadHandler.TO_BNF.handleRequest(grammar, null));

        System.out.println(RailroadHandler.BNF_TO_SVG.handleRequest(BNF, null));

        System.out.println(RailroadHandler.BNF_TO_SVG.handleRequest(SEM_VER, null));
    }
}
