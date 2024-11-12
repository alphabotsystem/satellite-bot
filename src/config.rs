use phf::{phf_map, Map};

pub const PROJECT: &str = "nlc-bot-36685";
pub const FREE_THRESHOLD: u32 = 10;
pub const REQUEST_REFRESH_SECONDS: u64 = 60 * 15;

pub static CONFIGURATION: Map<&'static str, (&'static str, Option<&'static str>, &'static str)> = phf_map! {
	"709850457467650138" => ("CCXT", Some("bitmex"), "BTCUSD"),
	"709891252530970711" => ("CCXT", Some("bitmex"), "XRPUSD"),
	"710074695495712788" => ("Twelvedata", None, "TSLA"),
	"710074767784280127" => ("Twelvedata", None, "AMD"),
	"710074859815698553" => ("Twelvedata", None, "NVDA"),
	"710074952153301022" => ("Twelvedata", None, "MSFT"),
	"710075001403080714" => ("Twelvedata", None, "AAPL"),
	"710075054356037663" => ("Twelvedata", None, "AMZN"),
	"738420614574112868" => ("CCXT", Some("bitmex"), "LTCUSD"),
	"738429177413500960" => ("CCXT", Some("binance"), "BTCUSD"),
	"738429689810518036" => ("CCXT", Some("binance"), "ETHUSD"),
	"739085555186532384" => ("CCXT", Some("binance"), "XRPUSD"),
	"739107100126478337" => ("CCXT", Some("binance"), "BCHUSD"),
	"739107866111377429" => ("CCXT", Some("binance"), "LTCUSD"),
	"739108704170803283" => ("CCXT", Some("binance"), "EOSUSD"),
	"739413924868522086" => ("CCXT", Some("binance"), "XLMUSD"),
	"743432577553006664" => ("CCXT", Some("bybit"), "BTCUSD"),
	"743433528964022383" => ("CCXT", Some("bybit"), "ETHUSD"),
	"743440072522727514" => ("CCXT", Some("htx"), "BTCUSD"),
	"743453643004575774" => ("CCXT", Some("binance"), "LINKUSD"),
	"743456887692984422" => ("CCXT", Some("binance"), "BNBUSD"),
	"743461822467932291" => ("CCXT", Some("binance"), "ADAUSD"),
	"745319169775632404" => ("CCXT", Some("gate"), "SRMUSD"),
	"745379499570495629" => ("CCXT", Some("binance"), "COMPUSD"),
	"745395371924127825" => ("CCXT", Some("binance"), "YFIUSD"),
	"751080162300526653" => ("CoinGecko", None, "BTCUSD"),
	"751080770243657779" => ("CoinGecko", None, "ETHUSD"),
	"751081142580412546" => ("CCXT", Some("binance"), "DOTUSDT"),
	"751081514178969670" => ("CCXT", Some("binance"), "HBARUSDT"),
	"751488841822634116" => ("Twelvedata", None, "GOOGL"),
	"751489005018677351" => ("Twelvedata", None, "NFLX"),
	"752207000728895590" => ("CoinGecko", None, "CROUSD"),
	"753250930022940702" => ("CCXT", Some("binance"), "VETUSDT"),
	"774377137515134996" => ("CoinGecko", None, "XJPUSD"),
	"786986778430799872" => ("Twelvedata", None, "NIO"),
	"786988356852383794" => ("Twelvedata", None, "PLTR"),
	"787004889019580436" => ("Twelvedata", None, "AQB"),
	"787714760074199080" => ("Twelvedata", None, "ROKU"),
	"787715515518156811" => ("Twelvedata", None, "FSLY"),
	"788087979671289866" => ("Twelvedata", None, "META"),
	"799586219285282816" => ("Twelvedata", None, "ALPP"),
	"799586397979410432" => ("Twelvedata", None, "GAXY"),
	"799586605106331719" => ("Twelvedata", None, "ABAT"),
	"799600802322186281" => ("CCXT", Some("binance"), "UNIUSD"),
	"799600878470299659" => ("CCXT", Some("binance"), "LRCUSDT"),
	"800307532366741504" => ("Twelvedata", None, "EURUSD"),
	"800307918242709504" => ("Twelvedata", None, "GBPUSD"),
	"802500457142157312" => ("CCXT", Some("binance"), "AAVEUSD"),
	"802649161928540200" => ("Twelvedata", None, "AUDJPY"),
	"802649221260902451" => ("Twelvedata", None, "AUDUSD"),
	"802649315100721192" => ("Twelvedata", None, "EURJPY"),
	"802649384193097779" => ("Twelvedata", None, "GBPJPY"),
	"802649469488201748" => ("Twelvedata", None, "NZDJPY"),
	"802649532705013801" => ("Twelvedata", None, "NZDUSD"),
	"802649628716564501" => ("Twelvedata", None, "USDCAD"),
	"802649693862887466" => ("Twelvedata", None, "USDJPY"),
	"802649746791596062" => ("Twelvedata", None, "USDZAR"),
	"805157874887819325" => ("Twelvedata", None, "GME"),
	"809728857573163059" => ("Twelvedata", None, "BOTY"),
	"809728957293002752" => ("CCXT", Some("binance"), "DOGEUSD"),
	"809729046661431306" => ("Twelvedata", None, "AMC"),
	"809729141112700968" => ("Twelvedata", None, "NOK"),
	"830786058165289000" => ("CCXT", Some("binance"), "ALGOUSDT"),
	"830830456433278986" => ("Twelvedata", None, "SPY"),
	"834009937242882079" => ("CoinGecko", None, "HNTUSD"),
	"837610492028780544" => ("Twelvedata", None, "COIN"),
	"837610857394995220" => ("CoinGecko", None, "SWISEUSD"),
	"837611716556357672" => ("CCXT", Some("kucoin"), "WAXPUSDT"),
	"837624778897883146" => ("CCXT", Some("binance"), "SOLUSD"),
	"837624891977105418" => ("CCXT", Some("binance"), "RAYUSD"),
	"842060571687780363" => ("CoinGecko", None, "TRIBEUSD"),
	"842061917555785728" => ("Twelvedata", None, "DIA"),
	"842317900034473986" => ("Twelvedata", None, "QQQ"),
	"842401105614471179" => ("Twelvedata", None, "FNKO"),
	"842401151815254057" => ("Twelvedata", None, "X"),
	"842401260710658098" => ("Twelvedata", None, "KIRK"),
	"842401341389406218" => ("Twelvedata", None, "PLBY"),
	"842401422087553115" => ("Twelvedata", None, "ASO"),
	"952620224476229632" => ("CCXT", Some("binance"), "XTZUSDT"),
	"952621342203715655" => ("CCXT", Some("binance"), "XTZBTC"),
	"978658777349910628" => ("Twelvedata", None, "F"),
	"979840523160408114" => ("CoinGecko", None, "OSETHUSD"),
	"1017784552929890395" => ("CCXT", Some("binance"), "LDOUSDT"),
	"1034089749247443064" => ("CCXT", Some("binance"), "APTUSDT"),
	"1092127867334316205" => ("Alternative.me", None, "FGI"),
	"1092128010242629754" => ("CNN Business", None, "FGI"),
	"1101942870832848906" => ("CoinGecko", None, "VRAUSD"),
	"1102222771280420904" => ("CCXT", Some("binance"), "PEPEUSD"),
	"1109849106773463193" => ("CoinGecko", None, "XMRUSD"),
	"1113175913870409738" => ("Twelvedata", None, "XAUUSD"),
	"1113176092296097854" => ("Twelvedata", None, "XAGUSD"),
	"1121112620439703723" => ("CCXT", Some("poloniex"), "BIGUSD"),
	"1121112828003238028" => ("Twelvedata", None, "NDX"),
	"1121112983305715923" => ("Twelvedata", None, "AEX:2"),
	"1130037256661512242" => ("Twelvedata", None, "HYLN"),
	"1161623718733299762" => ("Blockchair", None, "BTCUSD.HALVING"),
	"1166362320964943982" => ("CoinGecko", None, "IOCUSD"),
	"1205797816111210517" => ("CCXT", Some("binance"), "SEIUSDT"),
	"1213818330591531099" => ("CCXT", Some("binance"), "PEOPLEUSDT"),
	"1216094733223334028" => ("CCXT", Some("binance"), "MEMEUSD"),
	"1216094587215413368" => ("CCXT", Some("binance"), "WIFUSDT"),
	"1219014344583418009" => ("CoinGecko", None, "GUIUSD"),
	"1228027269675487232" => ("CoinGecko", None, "MYROUSD"),
	"1228028031306698834" => ("CoinGecko", None, "ZETAUSD"),
	"1228029687889133609" => ("CoinGecko", None, "MANEKIUSD"),
	"1234096647135559693" => ("CoinGecko", None, "BTCUSD.D"),
	"1234096714076520449" => ("CoinGecko", None, "ETHUSD.D"),
	"1234096758351462471" => ("CoinGecko", None, "SOLUSD.D"),
	"1235643059669172234" => ("CoinGecko", None, "USDTUSD.D"),
	"1238555265696268308" => ("CoinGecko", None, "BRETTUSD"),
	"1238607565676351529" => ("On-Chain", Some("base"), "0x3b933917c138cdd3d3ef6ec13223840c1007d075"),
	"1242439805657415731" => ("CCXT", Some("binance"), "ETHBTC"),
	"1242440601362894880" => ("CoinGecko", None, "WSDMUSD"),
	"1248640249492541480" => ("CoinGecko", None, "DRIFTUSD"),
	"1248644191534448690" => ("CoinGecko", None, "GMEUSD"),
	"1249095722758832138" => ("On-Chain", Some("base"), "0xa699c7ba0aee9f6f675d960aaaf1018e983c4247"),
	"1276573804730126430" => ("CCXT", Some("binance"), "TRXUSDT"),
	"1276576283165331626" => ("CoinGecko", None, "SUNUSD"),
	"1305842131251761152" => ("CCXT", Some("binance"), "INJUSDT"),
};