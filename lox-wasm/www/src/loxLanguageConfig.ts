export const registerLox = (monaco: any) => {
  monaco.languages.register({ id: "lox" });

  monaco.languages.setMonarchTokensProvider("lox", {
    tokenizer: {
      root: [
        [/\b(and|class|else|false|for|fun|if|nil|or|print|return|super|this|true|var|while)\b/, "keyword"],
        [/".*?"/, "string"],
        [/\d+(\.\d+)?/, "number"],
        [/\/\/.*$/, "comment"],
        [/[+\-*\/=!<>]=?|and|or/, "operator"],
        [/[a-zA-Z_]\w*/, "identifier"],
        [/[{}()]/, "delimiter"],
      ],
    },
  });

  monaco.languages.setLanguageConfiguration("lox", {
    comments: {
      lineComment: "//",
    },
    brackets: [
      ["{", "}"],
      ["(", ")"],
    ],
    autoClosingPairs: [
      { open: "{", close: "}" },
      { open: "(", close: ")" },
      { open: '"', close: '"' },
    ],
  });
};