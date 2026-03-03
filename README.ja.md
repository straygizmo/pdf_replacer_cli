# pdf_replacer_cli

PDFファイル内のテキストを一括置換するコマンドラインツールです。CSVファイルから置換ルールを読み込み、指定したPDFに適用します。

## 機能

- 単一ファイル・フォルダ一括処理に対応
- CSVベースの置換ルール（UTF-8 / Shift-JIS 自動判別）
- 日本語（CJK）テキスト置換対応（CIDフォント逆引きマッピング）
- デフォルトで元ファイルを保持する非破壊出力

## インストール

```bash
cargo build --release
```

バイナリは `target/release/pdf_replacer_cli.exe` に生成されます。

## 使い方

```
pdf_replacer_cli [OPTIONS] <INPUT>

引数:
  <INPUT>  PDFファイルパス または PDFを含むフォルダパス

オプション:
  -c, --csv <CSV>        CSVファイルパス [デフォルト: replacements.csv]
  -o, --output <OUTPUT>  出力先パス
      --in-place         元ファイルを上書き
  -v, --verbose          詳細ログ出力
  -h, --help             ヘルプ表示
```

### 実行例

```bash
# 単一ファイル処理（input_replaced.pdf として出力）
pdf_replacer_cli input.pdf

# フォルダ一括処理（./pdfs_replaced/ に出力）
pdf_replacer_cli ./pdfs/

# CSV・出力先を指定
pdf_replacer_cli input.pdf -c rules.csv -o output.pdf

# 詳細ログ付き
pdf_replacer_cli input.pdf -v

# 元ファイルを上書き
pdf_replacer_cli input.pdf --in-place
```

## CSV形式

カレントディレクトリに `replacements.csv` を配置してください。1行目はヘッダー行、2カラム構成です。

```csv
before,after
旧会社名,新会社名
置換前テキスト,置換後テキスト
```

UTF-8（BOM有無とも）および Shift-JIS（CP932）エンコーディングに対応しています。

## 制限事項

- テキストベースのPDFのみ対応（スキャン画像のPDFは不可）
- 置換後テキストは元フォントに含まれる文字のみ使用可能
- 暗号化PDFは完全にはサポートされていません
- 複数のPDFオペレータにまたがるテキストはマッチしない場合があります
