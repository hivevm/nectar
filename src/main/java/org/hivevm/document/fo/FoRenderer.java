package org.hivevm.document.fo;

import org.hivevm.doc.fo.Fo;
import org.hivevm.doc.fo.writer.*;
import org.hivevm.doc.template.*;
import org.hivevm.document.Document;
import org.hivevm.util.DataUri;
import org.hivevm.util.xml.StAX;
import org.hivevm.util.xml.XmlBuilder;

import javax.xml.stream.XMLStreamException;
import java.io.Closeable;
import java.io.IOException;
import java.io.OutputStream;
import java.util.Map;
import java.util.Objects;
import java.util.Properties;
import java.util.Stack;
import java.util.stream.Collectors;
import java.util.stream.Stream;

/// The FoWriter class provides a way to build and write XSL-FO documents to an output stream. It
/// utilizes an XmlBuilder to create the XML content and manages an FoRoot element as the root of
/// the XSL-FO document.
public class FoRenderer implements Closeable {

    private final Template template;
    private final XmlBuilder builder;
    private final FoRoot root;

    private final Stack<FoNode> nodes;
    private FoFlow flow;

    /// Creates a new instance of the FoWriter class to generate and write XSL-FO documents into the
    /// specified output stream. This constructor initializes the required builder with default
    /// formatting disabled.
    public FoRenderer(OutputStream stream, Template template) throws XMLStreamException {
        this(stream, template, false);
    }

    /// Constructs a FoWriter instance, which is used for generating and writing XSL-FO documents
    /// into the specified output stream. This constructor also allows specifying whether the
    /// generated XML content should be formatted for readability.
    public FoRenderer(OutputStream stream, Template template, boolean formatted)
            throws XMLStreamException {
        this.template = template;
        this.builder = new XmlBuilder(StAX.formatted(StAX.createWriter(stream), formatted ? 2 : 0));
        this.builder.addNamespace("fo", "http://www.w3.org/1999/XSL/Format");
        this.builder.addNamespace("fox", "http://xmlgraphics.apache.org/fop/extensions");
        this.builder.setNamespace("http://www.w3.org/1999/XSL/Format");
        this.root = new FoRoot(builder);
        this.root.setFontFamily(template.getFont());
        this.root.setFontSize("10pt");
        this.nodes = new Stack<>();
    }

    /// Creates and returns a new instance of FoLayoutMasterSet of the XSL-FO document being
    /// generated.
    public final FoLayoutMasterSet createLayoutMasteSet() {
        return new FoLayoutMasterSet(root);
    }

    public final FoRoot root() {
        return this.root;
    }

    public final Template template() {
        return this.template;
    }

    public final FoFlow flow() {
        return this.flow;
    }

    public final FoNode top() {
        return this.nodes.isEmpty() ? this.flow : this.nodes.peek();
    }

    public final FoNode pop() {
        return this.nodes.pop();
    }

    public final void push(FoNode node) {
        this.nodes.push(node);
    }

    public final void render(Document document) {
        renderLayout();
        DocumentRenderer.render(document, this);
    }

    /// Generates and configures an XSL-FO layout using the specified writer and template. This
    /// method constructs a layout master set, defines page masters with margins and sizes, and
    /// configures the regions and page sequences based on the template's properties.
    public final void renderLayout() {
        try (var layout = createLayoutMasteSet()) {
            for (var page : template().pages()) {
                var simple = layout.createPageMaster(page.getName()).setMarginTop(page.marginTop())
                        .setMarginLeft(page.marginLeft()).setMarginRight(page.marginRight())
                        .setMarginBottom(page.marginBottom())
                        .setPageSize(page.pageWidth(), page.pageHeight());

                try (var content = simple.createBodyRegion("region-body")) {
                    content.setMarginTop(
                                    Stream.of(page.paddingTop(), page.getRegion("top").getExtent())
                                            .filter(Objects::nonNull).collect(Collectors.joining(" + ")))
                            .setMarginLeft(
                                    Stream.of(page.paddingLeft(), page.getRegion("left").getExtent())
                                            .filter(Objects::nonNull).collect(Collectors.joining(" + ")))
                            .setMarginRight(
                                    Stream.of(page.paddingRight(), page.getRegion("right").getExtent())
                                            .filter(Objects::nonNull).collect(Collectors.joining(" + ")))
                            .setMarginBottom(
                                    Stream.of(page.paddingBottom(), page.getRegion("bottom").getExtent())
                                            .filter(Objects::nonNull).collect(Collectors.joining(" + ")))
                            .setColumns(page.columnCount(), page.columnGap());
                }

                page.forEachRegion(r -> (switch (r.getRegion()) {
                    case BEFORE -> simple.createRegionBefore(r.getName());
                    case START -> simple.createRegionStart(r.getName());
                    case END -> simple.createRegionEnd(r.getName());
                    case AFTER -> simple.createRegionAfter(r.getName());
                }).setExtent(r.getExtent()));
            }

            for (var s : template().pageSet()) {
                var master = layout.createPageSequence(s.getName());
                s.pages().forEach(e -> master.addPage(e.getKey().getName(), e.getValue()));
            }
        }
    }

    /**
     * Applies styling properties to the specified {@link FoBasicLink} element based on the
     * corresponding style configuration in the template.
     */
    public final void renderStyle(FoBasicLink link) {
        var style = template().getStyle("link");
        link.setColor(style.color);
        link.setFontSize(style.fontSize);
        link.setFontStyle(style.fontStyle);
        link.setFontWeight(style.fontWeight);
        link.setFontFamily(style.fontFamily);
        link.setTextAlign(style.textAlign);
        link.setLineHeight(style.lineHeight);
    }

    /**
     * Applies styling properties to the specified {@link FoBlock} element based on the
     * corresponding style configuration in the template.
     */
    public final void renderStyle(FoBlock block, String name) {
        PageStyle style = template().getStyle(name);

        block.setSpan(style.span);
        block.setColor(style.color);

        if (style.background != null) {
            if (style.background.isImage()) {
                block.setBackground(style.background.get(), "no-repeat");
            } else {
                block.setBackgroundColor(style.background.get());
            }
        }
        block.setFontSize(style.fontSize);
        block.setFontStyle(style.fontStyle);
        block.setFontWeight(style.fontWeight);
        block.setFontFamily(style.fontFamily);

        block.setTextAlign(style.textAlign);
        block.setLineHeight(style.lineHeight);
        block.setKeepWithNext(style.keepWithNext);

        if (style.spaceBefore != null) {
            String[] spaces = style.spaceBefore.split(",");
            if (spaces.length == 3) {
                block.setSpaceBefore(spaces[0], spaces[1], spaces[2]);
            } else {
                block.setSpaceBefore(style.spaceBefore);
            }
        }

        if (style.spaceAfter != null) {
            String[] spaces = style.spaceAfter.split(",");
            if (spaces.length == 3) {
                block.setSpaceAfter(spaces[0], spaces[1], spaces[2]);
            } else {
                block.setSpaceAfter(style.spaceAfter);
            }
        }

        block.setBorderRadius(style.borderRadius);
        if (style.borderTop != null) {
            String[] border = style.borderTop.split(" ");
            block.setBorderTop(border[0], border[1], border[2]);
        }
        if (style.borderLeft != null) {
            String[] border = style.borderLeft.split(" ");
            block.setBorderLeft(border[0], border[1], border[2]);
        }
        if (style.borderRight != null) {
            String[] border = style.borderRight.split(" ");
            block.setBorderRight(border[0], border[1], border[2]);
        }
        if (style.borderBottom != null) {
            String[] border = style.borderBottom.split(" ");
            block.setBorderBottom(border[0], border[1], border[2]);
        }

        block.setMarginTop(style.marginTop);
        block.setMarginLeft(style.marginLeft);
        block.setMarginRight(style.marginRight);
        block.setMarginBottom(style.marginBottom);

        block.setPaddingTop(style.paddingTop);
        block.setPaddingLeft(style.paddingLeft);
        block.setPaddingRight(style.paddingRight);
        block.setPaddingBottom(style.paddingBottom);

        block.setBreakBefore(style.breakBefore);
        block.setBreakAfter(style.breakAfter);
    }

    /**
     * Creates a new flow within a page sequence and configures it based on the provided parameters.
     * The flow is used to manage content within the specified page sequence and applies the
     * necessary formatting and layout rules according to the given sequence and document
     * requirements.
     */
    public final void createFlow(String id, String name, Properties properties) {
        FoPageSequence sequence = root().addPageSequence(id).setReference(name).setLanguage("en");
        if (name.equals(Fo.PAGESET_BOOK)) {
            sequence.setFormat("I").setInitialPageNumber("1");
        } else if (name.equals(Fo.PAGESET_CHAPTER)) {
            sequence.setFormat("1").setInitialPageNumber("auto");
        } else {
            sequence.setFormat("I").setInitialPageNumber("1");
        }

        FoStaticContent content = new FoStaticContent(sequence);
        content.setFlowName("xsl-footnote-separator");

        FoBlock block = content.addBlock().setTextAlignLast("justify").setPadding("0.5em");

        new FoLeader(block).setPattern("rule").setLength("50%").setRuleThickness("0.5pt")
                .setColor("#777777");

        // Gets the page set for this page flow
        for (PageSet pageSet : template().pageSet()) {
            if (!pageSet.getName().equals(name)) {
                continue;
            }

            for (Map.Entry<Page, PageMatch> e : pageSet.pages()) {
                var page = e.getKey();
                page.forEachRegion(r -> renderRegion(page, r, sequence, properties));
            }
        }

        this.flow = sequence.flow();
        this.flow.setName("region-body");
        this.flow.setStartIndent("0pt").setEndIndent("0pt");

        this.nodes.clear();
    }

    /// Closes this {@code FoWriter} instance and releases any underlying resources associated with
    /// it. This involves closing the FoRoot and XmlBuilder instances managed by this writer.
    @Override
    public final void close() throws IOException {
        this.root.close();
        this.builder.close();
    }

    /**
     * Renders a specified region of a page within a FO page sequence. This method processes the
     * region's content and configures layout, including handling watermarks and static images for
     * specific regions.
     */
    private static void renderRegion(Page page, PageRegion region, FoPageSequence node,
                                     Properties properties) {
        if (region.hasChildren() || (region.getRegion() == PageRegion.Region.BEFORE
                && page.background != null)) {
            FoStaticContent content = new FoStaticContent(node);
            content.setFlowName(region.getName());

            // Render Page fixed layouts (Watermarks)
            if (region.getRegion() == PageRegion.Region.BEFORE) {
                if (page.background != null) {
                    renderWatermark(page, content);
                }

                page.items.stream().filter(i -> i instanceof PageImage).map(i -> (PageImage) i)
                        .forEach(i -> {
                            FoBlockContainer container = content.blockContainer();
                            container.setAbsolute(FoBlockContainer.Position.Fixed)
                                    .setPositionTop(i.getTop()).setPositionLeft(i.getLeft())
//                                    .setPositionRight(i.getRight())
//                                    .setPositionBottom(i.getBottom())
                                    .setWidth(i.getWidth()).setHeight(i.getHeight());

                            FoBlock block = container.addBlock();

                            FoExternalGraphic image = new FoExternalGraphic(block);
                            image.setURL(DataUri.loadImage(i.uri));
                            image.setSize(i.getWidth(), i.getHeight());
                            image.setContentWidth("scale-to-fit");
                            image.setContentHeight("scale-to-fit");
                        });
            }

            region.forEachItem(p -> renderAbsolute(p, content, properties));
        }
    }

    /**
     * Renders the specified {@code PageColumn} as an absolute-positioned block within the provided
     * {@code FoStaticContent} node. The method uses styling and layout properties defined in the
     * {@code PageColumn} to configure the FO block container.
     */
    private static void renderAbsolute(PageColumn panel, FoStaticContent node,
                                       Properties properties) {
        FoBlockContainer container = node.blockContainer();
        container.setAbsolute(FoBlockContainer.Position.Absolute)
                .setPosition(panel.getLeft(), panel.getRight(), panel.getTop(), panel.getBottom());

        container.setColor(panel.color);
        if (panel.background != null) {
            if (panel.background.isImage()) {
                container.setBackground(panel.background.get(), "no-repeat");
            } else {
                container.setBackgroundColor(panel.background.get());
            }
        }
        container.setFontSize(panel.fontSize);
        container.setFontStyle(panel.fontStyle);
        container.setFontWeight(panel.fontWeight);
        container.setFontFamily(panel.fontFamily);
        container.setTextAlign(panel.textAlign);
        container.setLineHeight(panel.lineHeight);

        PageRenderer renderer = new PageRenderer(properties);
        panel.items.forEach(c -> c.accept(renderer, container.addBlock()));
    }

    /**
     * Renders a watermark on the specified page by configuring the static content with a block
     * container and optional background imagery or color.
     */
    private static void renderWatermark(Page page, FoStaticContent content) {
        FoBlockContainer container = content.blockContainer();
        container.setAbsolute(FoBlockContainer.Position.Fixed).setPositionTop("0")
                .setPositionLeft("0").setPositionRight("0").setPositionBottom("0");

        if (!page.background.isImage()) {
            container.setBackgroundColor(page.background.get());
        }

        FoBlock block = container.addBlock();

        if (page.background.isImage()) {
            FoExternalGraphic watermark = new FoExternalGraphic(block);
            watermark.setURL(page.background.get());
            watermark.setContentWidth(page.pageWidth());
            watermark.setContentHeight(page.pageHeight());
        }
    }
}
