# Wiki Pages

This directory contains markdown files for the UMAP Text Visualizer GitHub Wiki.

## Contents

- **Home.md** - Wiki home page with overview and quick links
- **Architecture.md** - System architecture with Mermaid diagrams
- **Data-Flow.md** - Sequence diagrams and data flow patterns
- **umap-core.md** - Core library component details
- **umap-cli.md** - CLI and server component details
- **umap-web.md** - Frontend component details
- **Deployment.md** - Deployment guides for various platforms
- **_Sidebar.md** - Navigation sidebar

## Link Formats Used

### Wiki Page to Wiki Page Links

Format: `[[Page Name]]` or `[[Link Text|Page Name]]`

Examples:
- `[[Architecture]]` - Links to Architecture wiki page
- `[[Link Text|umap-core]]` - Links to umap-core page with custom text

### Wiki Page to Repository File Links

Format: `[Link Text](../../blob/main/path/to/file.md)` or `[Link Text](../../tree/main/path/to/dir)`

Examples:
- `[Development Process](../../blob/main/documentation/process.md)` - Links to process.md in repo
- `[Source Code](../../tree/main/crates/umap-core)` - Links to directory in repo

### Sidebar Links

The `_Sidebar.md` uses the same format as wiki-to-wiki links for wiki pages, and relative paths for repository files.

## Uploading to GitHub Wiki

GitHub Wikis are stored in a separate Git repository. You can clone and manage it like any other repository.

### Method 1: Clone Wiki Repository

```bash
# Clone the wiki repository
git clone https://github.com/sw-viz/umap.wiki.git

# Copy wiki files
cp wiki/*.md umap.wiki/

# Commit and push
cd umap.wiki
git add .
git commit -m "Add comprehensive architecture documentation with diagrams"
git push origin master
```

### Method 2: Manual Upload via GitHub UI

1. Go to https://github.com/sw-viz/umap/wiki
2. Click "Create new page" or edit existing pages
3. Copy content from each .md file
4. Save each page

**Note:** The sidebar requires special handling:
- The `_Sidebar.md` file must be named exactly `_Sidebar`
- It will appear on all wiki pages automatically

### Method 3: Use GitHub API

```bash
# First time: Clone wiki repo
git clone https://github.com/sw-viz/umap.wiki.git wiki-repo

# Update script
#!/bin/bash
cd wiki-repo

# Copy all wiki files
cp ../wiki/*.md ./

# Commit and push
git add .
git commit -m "Update wiki documentation"
git push origin master
```

## Verifying Wiki

After uploading, verify:

1. **Navigation works**
   - Sidebar appears on all pages
   - Links between wiki pages work
   - Links to repository files work

2. **Mermaid diagrams render**
   - Architecture diagrams display correctly
   - Sequence diagrams display correctly
   - Graph diagrams display correctly

3. **Formatting is correct**
   - Code blocks render properly
   - Tables display correctly
   - Headers create proper hierarchy

## Maintaining Wiki

When updating documentation:

1. Update the corresponding .md file in the `wiki/` directory
2. Test locally (use a Markdown previewer with Mermaid support)
3. Commit changes to main repository
4. Clone wiki repository and copy updated files
5. Push to wiki repository

## Mermaid Diagram Notes

GitHub Wiki supports Mermaid diagrams natively. The diagrams use:

- `graph TD` - Top-down flowcharts
- `graph LR` - Left-right flowcharts
- `sequenceDiagram` - Sequence diagrams
- `erDiagram` - Entity-relationship diagrams

**Important:**
- No HTML break elements (`<br>`) in Mermaid diagrams
- Use descriptive node IDs
- Keep diagrams focused and readable

## Troubleshooting

### Links not working

- **Wiki-to-wiki links:** Ensure format is `[[Page Name]]` without .md extension
- **Wiki-to-repo links:** Ensure relative path starts with `../../`
- **Spaces in page names:** GitHub converts to hyphens in URLs (e.g., "Data Flow" → "Data-Flow")

### Diagrams not rendering

- Check for syntax errors in Mermaid code
- Ensure code blocks use triple backticks with `mermaid` language tag
- No HTML or special characters in diagram definitions

### Sidebar not appearing

- File must be named exactly `_Sidebar.md` (with underscore)
- Must be in root of wiki repository
- Check that file was successfully pushed

## Additional Resources

- [GitHub Wiki Documentation](https://docs.github.com/en/communities/documenting-your-project-with-wikis)
- [Mermaid Documentation](https://mermaid.js.org/)
- [Markdown Guide](https://www.markdownguide.org/)
