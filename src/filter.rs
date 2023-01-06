pub struct Filter {
    forbidden_words: Vec<String>,
}

//filters out forbidden words from input string
impl Filter {
    pub fn new(forbidden_words: &[String]) -> Self {
        Self {
            forbidden_words: forbidden_words.to_owned(),
        }
    }

    pub fn is_normal(&self, s: &str) -> bool {
        !self.forbidden_words.iter().any(|w| s.contains(w))
    }

    pub fn sanitize(&self, s: &str) -> String {
        let mut ret = s.to_string();
        for w in &self.forbidden_words {
            if (ret.contains(w)) {
                ret = ret.replace(w, &"*".repeat(w.chars().count()));
            }
        }
        ret
    }
}

#[cfg(test)]
mod tests {
    use std::time::Instant;

    use super::*;

    #[test]
    // #[ignore]
    fn test01() {
        let filter = Filter::new(&vec![]);
        assert_eq!(true, filter.is_normal("あいうえお"));
    }

    #[test]
    // #[ignore]
    fn test02() {
        let filter = Filter::new(&vec!["りんご".to_string(), "ゴリラ".to_string()]);
        assert_eq!(true, filter.is_normal("あいうえお"));
        assert_eq!(false, filter.is_normal("ありんごい"));
        assert_eq!(false, filter.is_normal("あゴリラい"));
        assert_eq!(false, filter.is_normal("あゴリラりんごい"));
    }

    #[test]
    // #[ignore]
    fn test03() {
        let filter = Filter::new(&vec!["りんご".to_string(), "ボックス".to_string()]);
        assert_eq!("あいうえお".to_string(), filter.sanitize("あいうえお"));
        assert_eq!("あ***い".to_string(), filter.sanitize("ありんごい"));
        assert_eq!(
            "あ******い".to_string(),
            filter.sanitize("ありんごりんごい")
        );
        assert_eq!("あ****い".to_string(), filter.sanitize("あボックスい"));
        assert_eq!("あ****う***い", filter.sanitize("あボックスうりんごい"));
    }

    //performance test
    #[test]
    // #[ignore]
    fn test04() {
        //forbidden words are cited from |https://ryoko-club.com/food/|
        let filter = Filter::new(&vec![
            "柿".to_string(),
            "桃".to_string(),
            "梅".to_string(),
            "梨".to_string(),
            "鮎".to_string(),
            "鮭".to_string(),
            "鯉".to_string(),
            "鯖".to_string(),
            "鯛".to_string(),
            "うど".to_string(),
            "うに".to_string(),
            "かぶ".to_string(),
            "しそ".to_string(),
            "せり".to_string(),
            "たこ".to_string(),
            "たら".to_string(),
            "ねぎ".to_string(),
            "びわ".to_string(),
            "ふき".to_string(),
            "ふぐ".to_string(),
            "ぶり".to_string(),
            "ほや".to_string(),
            "ゆず".to_string(),
            "アジ".to_string(),
            "イカ".to_string(),
            "エビ".to_string(),
            "カニ".to_string(),
            "ナス".to_string(),
            "ニラ".to_string(),
            "ノニ".to_string(),
            "ハム".to_string(),
            "マカ".to_string(),
            "人参".to_string(),
            "冬瓜".to_string(),
            "大根".to_string(),
            "春菊".to_string(),
            "枝豆".to_string(),
            "水菜".to_string(),
            "洋梨".to_string(),
            "牛肉".to_string(),
            "牡蠣".to_string(),
            "生姜".to_string(),
            "白菜".to_string(),
            "砂肝".to_string(),
            "穴子".to_string(),
            "豆苗".to_string(),
            "豚肉".to_string(),
            "豚足".to_string(),
            "赤貝".to_string(),
            "金柑".to_string(),
            "馬肉".to_string(),
            "高菜".to_string(),
            "鶏肉".to_string(),
            "あけび".to_string(),
            "あさり".to_string(),
            "あわび".to_string(),
            "あんず".to_string(),
            "いくら".to_string(),
            "いちご".to_string(),
            "うなぎ".to_string(),
            "かつお".to_string(),
            "かぼす".to_string(),
            "かます".to_string(),
            "かりん".to_string(),
            "くわい".to_string(),
            "こごみ".to_string(),
            "ごぼう".to_string(),
            "さわら".to_string(),
            "さんま".to_string(),
            "ざくろ".to_string(),
            "しじみ".to_string(),
            "すだち".to_string(),
            "すもも".to_string(),
            "ずいき".to_string(),
            "せとか".to_string(),
            "そら豆".to_string(),
            "たらこ".to_string(),
            "つくし".to_string(),
            "なつめ".to_string(),
            "なまこ".to_string(),
            "にぼし".to_string(),
            "ぶどう".to_string(),
            "へちま".to_string(),
            "ほっけ".to_string(),
            "みかん".to_string(),
            "もやし".to_string(),
            "よもぎ".to_string(),
            "りんご".to_string(),
            "わらび".to_string(),
            "アロエ".to_string(),
            "イワシ".to_string(),
            "ウコン".to_string(),
            "オクラ".to_string(),
            "カジキ".to_string(),
            "カレイ".to_string(),
            "ガラナ".to_string(),
            "キウイ".to_string(),
            "キムチ".to_string(),
            "グァバ".to_string(),
            "ケール".to_string(),
            "ゴーヤ".to_string(),
            "サザエ".to_string(),
            "サラミ".to_string(),
            "スイカ".to_string(),
            "セロリ".to_string(),
            "チコリ".to_string(),
            "トマト".to_string(),
            "ニシン".to_string(),
            "ノビル".to_string(),
            "バジル".to_string(),
            "バナナ".to_string(),
            "パセリ".to_string(),
            "ヒラメ".to_string(),
            "ビーツ".to_string(),
            "ホタテ".to_string(),
            "ポポー".to_string(),
            "マグロ".to_string(),
            "ミント".to_string(),
            "メロン".to_string(),
            "メンマ".to_string(),
            "ユリ根".to_string(),
            "ライチ".to_string(),
            "ライム".to_string(),
            "ラム肉".to_string(),
            "レタス".to_string(),
            "レバー".to_string(),
            "レモン".to_string(),
            "三つ葉".to_string(),
            "四角豆".to_string(),
            "太刀魚".to_string(),
            "小松菜".to_string(),
            "梅干し".to_string(),
            "玉ねぎ".to_string(),
            "生ハム".to_string(),
            "空芯菜".to_string(),
            "菜の花".to_string(),
            "蜂の子".to_string(),
            "野沢菜".to_string(),
            "金時草".to_string(),
            "金目鯛".to_string(),
            "長命草".to_string(),
            "食用菊".to_string(),
            "鶏胸肉".to_string(),
            "あしたば".to_string(),
            "あんこう".to_string(),
            "いちじく".to_string(),
            "いよかん".to_string(),
            "かぼちゃ".to_string(),
            "からし菜".to_string(),
            "からすみ".to_string(),
            "きびなご".to_string(),
            "きゅうり".to_string(),
            "ししとう".to_string(),
            "ししゃも".to_string(),
            "すっぽん".to_string(),
            "ぜんまい".to_string(),
            "たけのこ".to_string(),
            "たらの芽".to_string(),
            "とんぶり".to_string(),
            "にんにく".to_string(),
            "ぬか漬け".to_string(),
            "のどぐろ".to_string(),
            "はっさく".to_string(),
            "はまぐり".to_string(),
            "みょうが".to_string(),
            "アサイー".to_string(),
            "アセロラ".to_string(),
            "アピオス".to_string(),
            "アボカド".to_string(),
            "オリーブ".to_string(),
            "オレンジ".to_string(),
            "カムカム".to_string(),
            "カワハギ".to_string(),
            "カンパチ".to_string(),
            "キャビア".to_string(),
            "キャベツ".to_string(),
            "クコの実".to_string(),
            "クレソン".to_string(),
            "サンチュ".to_string(),
            "ザーサイ".to_string(),
            "ソルダム".to_string(),
            "タアサイ".to_string(),
            "デコポン".to_string(),
            "ドジョウ".to_string(),
            "ドリアン".to_string(),
            "ハタハタ".to_string(),
            "パクチー".to_string(),
            "パパイヤ".to_string(),
            "パプリカ".to_string(),
            "ピクルス".to_string(),
            "ピーマン".to_string(),
            "フカヒレ".to_string(),
            "ブンタン".to_string(),
            "プルーン".to_string(),
            "ポンカン".to_string(),
            "マンゴー".to_string(),
            "ムール貝".to_string(),
            "ヤマモモ".to_string(),
            "ルッコラ".to_string(),
            "ルバーブ".to_string(),
            "レンコン".to_string(),
            "レーズン".to_string(),
            "ワカサギ".to_string(),
            "夏みかん".to_string(),
            "田七人参".to_string(),
            "紫玉ねぎ".to_string(),
            "高麗人参".to_string(),
            "おかひじき".to_string(),
            "おかわかめ".to_string(),
            "かんぴょう".to_string(),
            "さくらんぼ".to_string(),
            "しらす干し".to_string(),
            "じゅんさい".to_string(),
            "ふきのとう".to_string(),
            "ほうれん草".to_string(),
            "らっきょう".to_string(),
            "エスカルゴ".to_string(),
            "ココナッツ".to_string(),
            "コールラビ".to_string(),
            "スプラウト".to_string(),
            "ズッキーニ".to_string(),
            "ソーセージ".to_string(),
            "タマリンド".to_string(),
            "チンゲン菜".to_string(),
            "ネクタリン".to_string(),
            "ハスカップ".to_string(),
            "ハヤトウリ".to_string(),
            "フェンネル".to_string(),
            "フォアグラ".to_string(),
            "フダンソウ".to_string(),
            "マキベリー".to_string(),
            "マコモダケ".to_string(),
            "マスカット".to_string(),
            "マルベリー".to_string(),
            "ミニトマト".to_string(),
            "モロヘイヤ".to_string(),
            "ラズベリー".to_string(),
            "大根おろし".to_string(),
            "紫キャベツ".to_string(),
            "芽キャベツ".to_string(),
            "かいわれ大根".to_string(),
            "さやいんげん".to_string(),
            "さやえんどう".to_string(),
            "つるむらさき".to_string(),
            "とうもろこし".to_string(),
            "にんにくの芽".to_string(),
            "アスパラガス".to_string(),
            "エシャレット".to_string(),
            "エシャロット".to_string(),
            "カリフラワー".to_string(),
            "クランベリー".to_string(),
            "コーンビーフ".to_string(),
            "パイナップル".to_string(),
            "フェイジョア".to_string(),
            "ブルーベリー".to_string(),
            "ブロッコリー".to_string(),
            "マンゴスチン".to_string(),
            "メロゴールド".to_string(),
            "ヤングコーン".to_string(),
            "ラディッシュ".to_string(),
            "切り干し大根".to_string(),
            "行者にんにく".to_string(),
            "ちりめんじゃこ".to_string(),
            "アイスプラント".to_string(),
            "グリーンピース".to_string(),
            "シークワーサー".to_string(),
            "スウィーティー".to_string(),
            "スターフルーツ".to_string(),
            "タイガーナッツ".to_string(),
            "ドライフルーツ".to_string(),
            "ブラックベリー".to_string(),
            "ホンビノスガイ".to_string(),
            "ローストビーフ".to_string(),
            "そうめんかぼちゃ".to_string(),
            "アーティチョーク".to_string(),
            "グレープフルーツ".to_string(),
            "ゴールデンベリー".to_string(),
            "ジャックフルーツ".to_string(),
            "スナップエンドウ".to_string(),
            "ドラゴンフルーツ".to_string(),
            "ビーフジャーキー".to_string(),
            "アメリカンチェリー".to_string(),
            "パッションフルーツ".to_string(),
            "ブロッコリースーパースプラウト".to_string(),
        ]);
        let start = Instant::now();
        filter.sanitize("ドライフルーツにんにくの芽大根おろし大根おろしラズベリーミニトマトしそエビ馬肉あけびラディッシュそうめんかぼちゃパッションフルーツゴールデンベリー大豆桃人参モロヘイヤ砂肝ほっけゴーヤヒラメパセリごぼうしじみたらこなつめなまこみかんアロエ-ドライフルーツにんにくの芽大根おろし大根おろしラズベリーミニトマトしそエビ馬肉あけびラディッシュそうめんかぼちゃパッションフルーツゴールデンベリー大豆桃人参モロヘイヤ砂肝ほっけゴーヤヒラメパセリごぼうしじみたらこなつめなまこみかんアロエ-ドライフルーツにんにくの芽大根おろし大根おろしラズベリーミニトマトしそエビ馬肉あけびラディッシュそうめんかぼちゃパッションフルーツゴールデンベリー大豆桃人参モロヘイヤ砂肝ほっけゴーヤヒラメパセリごぼうしじみたらこなつめなまこみかんアロエ");
        println!("{:?}", start.elapsed());
    }
}
