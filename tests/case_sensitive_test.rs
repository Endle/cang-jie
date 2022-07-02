use cang_jie::{CangJieTokenizer, TokenizerOption, CANG_JIE};
use flexi_logger::{Logger};
use jieba_rs::Jieba;
use std::{collections::HashSet, iter::FromIterator, sync::Arc};
use std::collections::hash_map::RandomState;
use tantivy::{collector::TopDocs, doc, query::QueryParser, schema::*, Index, Score, LeasedItem, Searcher};

#[test]
fn case_sensitive() -> tantivy::Result<()> {
    Logger::try_with_env_or_str("cang_jie=trace,error")
        .expect("failed to init logger")
        .start()
        .unwrap_or_else(|e| panic!("Logger initialization failed with {}", e));

    let mut schema_builder = SchemaBuilder::default();

    let text_indexing = TextFieldIndexing::default()
        .set_tokenizer(CANG_JIE) // Set custom tokenizer
        .set_index_option(IndexRecordOption::WithFreqsAndPositions);
    let text_options = TextOptions::default()
        .set_indexing_options(text_indexing)
        .set_stored();

    let title = schema_builder.add_text_field("title", text_options);
    let schema = schema_builder.build();

    let index = Index::create_in_ram(schema);
    index.tokenizers().register(CANG_JIE, tokenizer()); // Build cang-jie Tokenizer

    let mut index_writer = index.writer(50 * 1024 * 1024)?;
    index_writer.add_document(doc! { title => "Hello World" });
    index_writer.add_document(doc! { title => "hello world" });
    index_writer.commit()?;

    let reader = index.reader()?;
    let searcher = reader.searcher();

    let query = QueryParser::for_index(&index, vec![title]).parse_query("Hello")?;
    let top_docs = searcher.search(query.as_ref(), &TopDocs::with_limit(10000))?;

    let actual = result_to_vec(top_docs, &searcher, title);

    let expect = HashSet::from_iter(vec!["Hello World".to_string()]);

    assert_eq!(actual, expect);

    Ok(())
}

fn result_to_vec(top_docs: Vec<(Score, tantivy::DocAddress)>,
                 searcher: &LeasedItem<Searcher>, title: Field)
-> HashSet<String, RandomState>{
    let actual = top_docs
        .into_iter()
        .map(|x| {
            searcher
                .doc(x.1)
                .unwrap()
                .get_first(title)
                .unwrap()
                .text()
                .unwrap()
                .to_string()
        })
        .collect::<HashSet<_>>();
    actual
}
fn tokenizer() -> CangJieTokenizer {
    CangJieTokenizer {
        worker: Arc::new(Jieba::empty()), // empty dictionary
        option: TokenizerOption::Unicode,
    }
}
