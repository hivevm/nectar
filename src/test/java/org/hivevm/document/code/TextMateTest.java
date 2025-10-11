package org.hivevm.document.code;

import org.eclipse.tm4e.core.grammar.IGrammar;
import org.eclipse.tm4e.core.registry.IGrammarSource;
import org.eclipse.tm4e.core.registry.Registry;
import org.hivevm.util.text2svg.TextMateRenderer;
import org.junit.jupiter.api.Test;

import java.util.regex.Pattern;

public class TextMateTest {

    private static final String GRAMMAR_CPP = "/grammars/cpp.json";
    private static final String GRAMMAR_INI = "/grammars/ini.json";
    private static final String GRAMMAR_JAVA = "/grammars/java.json";
    private static final String GRAMMAR_YAML = "/grammars/yaml.json";

    private static final String test = """
            ; last modified 1 April 2001 by John Doe
            [owner]
            name = John Doe
            organization = Acme Widgets Inc.
            
            [database]
            ; use IP address in case network name resolution is not working
            server = 192.0.2.62    \s
            port = 143
            file = "payroll.dat"
            """;

    private static final String test1 = """
            [owner]
            name = John Doe
            organization = Acme Widgets Inc.
            
            [database]
            ; use IP address in case network name resolution is not working
            server = 192.0.2.62    \s
            port = 143
            file = "payroll.dat"
            """;

    private IGrammar loadGrammar(String type) {
        var registry = new Registry();
        registry.addGrammar(IGrammarSource.fromResource(TextMateRenderer.class, GRAMMAR_INI));
        return registry.grammarForScopeName("source." + type);
    }

    @Test
    public void testIniGrammar() throws Exception {
        var lineTokens = loadGrammar("ini").tokenizeLine(test1);

    }

    @Test
    public void testRegExp() throws Exception {
        var pattern = Pattern.compile("^(\\[)(.*?)(])");
        var matcher = pattern.matcher(test.substring(41));
        System.out.println(matcher.find());
    }
}
