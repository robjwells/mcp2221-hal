# Generate the CLI man page from the markdown source.
man:
	pandoc --standalone --to man mcp2221-cli/doc/mcp2221-cli.1.md -o mcp2221-cli/doc/mcp2221-cli.1
