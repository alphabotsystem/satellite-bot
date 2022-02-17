satellites = [
	751080162300526653, 751080770243657779, 709850457467650138, 709891252530970711, 738420614574112868, 738429177413500960, 738429689810518036, 739085555186532384, 739107100126478337, 739107866111377429, 743453643004575774, 743461822467932291, 743456887692984422, 739108704170803283, 739413924868522086, 743440072522727514, 743432577553006664, 745364520280522894, 743433528964022383, 745319169775632404, 745379499570495629, 745395371924127825, 751081142580412546, 752207000728895590, 751081514178969670, 753250930022940702, 751085756914728980, 774377137515134996, 710075001403080714, 710074695495712788, 710074767784280127, 710074859815698553, 710074952153301022, 710075054356037663, 751488841822634116, 751489005018677351, 786986778430799872, 786988356852383794, 787004889019580436, 787714760074199080, 787715515518156811, 788087979671289866, 799586219285282816, 799586397979410432, 799586549187870720, 799586605106331719, 799600802322186281, 799600878470299659, 800307532366741504, 800307918242709504, 802500457142157312, 802649161928540200, 802649221260902451, 802649315100721192, 802649384193097779, 802649469488201748, 802649532705013801, 802649628716564501, 802649693862887466, 802649746791596062, 802860366052458516, 805157874887819325, 809728857573163059, 809728957293002752, 809729046661431306, 809729141112700968, 830786058165289000, 830830456433278986, 834009937242882079, 837610492028780544, 837610638658109470, 837610857394995220, 837611716556357672, 837624778897883146, 837624891977105418, 842060498223628289, 842060571687780363, 842061917555785728, 842317900034473986, 842401105614471179, 842401151815254057, 842401260710658098, 842401341389406218, 842401422087553115
]
configuration = {
	709850457467650138: ["CCXT", "bitmex", "BTCUSD"],
	709891252530970711: ["CCXT", "bitmex", "XRPUSD"],
	710074695495712788: ["IEXC", None, "TSLA"],
	710074767784280127: ["IEXC", None, "AMD"],
	710074859815698553: ["IEXC", None, "NVDA"],
	710074952153301022: ["IEXC", None, "MSFT"],
	710075001403080714: ["IEXC", None, "AAPL"],
	710075054356037663: ["IEXC", None, "AMZN"],
	738420614574112868: ["CCXT", "bitmex", "LTCUSD"],
	738429177413500960: ["CCXT", "binance", "BTCUSD"],
	738429689810518036: ["CCXT", "binance", "ETHUSD"],
	739085555186532384: ["CCXT", "binance", "XRPUSD"],
	739107100126478337: ["CCXT", "binance", "BCHUSD"],
	739107866111377429: ["CCXT", "binance", "LTCUSD"],
	739108704170803283: ["CCXT", "binance", "EOSUSD"],
	739413924868522086: ["CCXT", "binance", "XLMUSD"],
	743432577553006664: ["CCXT", "ftx", "BTCUSD"],
	743433528964022383: ["CCXT", "ftx", "ETHUSD"],
	743440072522727514: ["CCXT", "huobi", "BTCUSD"],
	743453643004575774: ["CCXT", "binance", "LINKUSD"],
	743456887692984422: ["CCXT", "binance", "BNBUSD"],
	743461822467932291: ["CCXT", "binance", "ADAUSD"],
	745319169775632404: ["CCXT", "ftx", "SRMUSD"],
	745364520280522894: ["CCXT", "ftx", "BTCMOVE"],
	745379499570495629: ["CCXT", "ftx", "COMPUSD"],
	745395371924127825: ["CCXT", "ftx", "YFIUSD"],
	751080162300526653: ["CoinGecko", None, "BTCUSD"],
	751080770243657779: ["CoinGecko", None, "ETHUSD"],
	751081142580412546: ["CoinGecko", None, "DOTUSD"],
	751081514178969670: ["CoinGecko", None, "HBARUSD"],
	751085756914728980: ["CoinGecko", None, "OCEUSD"],
	751488841822634116: ["IEXC", None, "GOOGL"],
	751489005018677351: ["IEXC", None, "NFLX"],
	752207000728895590: ["CoinGecko", None, "CROUSD"],
	753250930022940702: ["CoinGecko", None, "VETUSD"],
	774377137515134996: ["CoinGecko", None, "XJPUSD"],
	786986778430799872: ["IEXC", None, "NIO"],
	786988356852383794: ["IEXC", None, "PLTR"],
	787004889019580436: ["IEXC", None, "AQB"],
	787714760074199080: ["IEXC", None, "ROKU"],
	787715515518156811: ["IEXC", None, "FSLY"],
	788087979671289866: ["IEXC", None, "FB"],
	799586219285282816: ["IEXC", None, "ALPP"],
	799586397979410432: ["IEXC", None, "GAXY"],
	799586549187870720: ["IEXC", None, "ARBKF"],
	799586605106331719: ["IEXC", None, "ABML"],
	799600802322186281: ["CoinGecko", None, "UNIUSD"],
	799600878470299659: ["CoinGecko", None, "LRCUSD"],
	800307532366741504: ["IEXC", None, "EURUSD"],
	800307918242709504: ["IEXC", None, "GBPUSD"],
	802500457142157312: ["CoinGecko", None, "AAVEUSD"],
	802649161928540200: ["IEXC", None, "AUDJPY"],
	802649221260902451: ["IEXC", None, "AUDUSD"],
	802649315100721192: ["IEXC", None, "EURJPY"],
	802649384193097779: ["IEXC", None, "GBPJPY"],
	802649469488201748: ["IEXC", None, "NZDJPY"],
	802649532705013801: ["IEXC", None, "NZDUSD"],
	802649628716564501: ["IEXC", None, "USDCAD"],
	802649693862887466: ["IEXC", None, "USDJPY"],
	802649746791596062: ["IEXC", None, "USDZAR"],
	802860366052458516: ["CoinGecko", None, "CRVUSD"],
	805157874887819325: ["IEXC", None, "GME"],
	809728857573163059: ["IEXC", None, "BOTY"],
	809728957293002752: ["CCXT", "binance", "DOGEUSD"],
	809729046661431306: ["IEXC", None, "AMC"],
	809729141112700968: ["IEXC", None, "NOK"],
	830786058165289000: ["CoinGecko", None, "ALGOUSD"],
	830830456433278986: ["IEXC", None, "SPY"],
	834009937242882079: ["CoinGecko", None, "HNTUSD"],
	837610492028780544: ["IEXC", None, "COIN"],
	837610638658109470: ["CCXT", "ftx", "FTTUSD"],
	837610857394995220: ["CoinGecko", None, "SWISEUSD"],
	837611716556357672: ["CCXT", "kucoin", "WAXPUSDT"],
	837624778897883146: ["CCXT", "ftx", "SOLUSD"],
	837624891977105418: ["CCXT", "ftx", "RAYUSD"],
	842060498223628289: ["CCXT", "ftx", "COPEUSD"],
	842060571687780363: ["CoinGecko", None, "TRIBEUSD"],
	842061917555785728: ["IEXC", None, "DIA"],
	842317900034473986: ["IEXC", None, "QQQ"],
	842401105614471179: ["IEXC", None, "FNKO"],
	842401151815254057: ["IEXC", None, "X"],
	842401260710658098: ["IEXC", None, "KIRK"],
	842401341389406218: ["IEXC", None, "PLBY"],
	842401422087553115: ["IEXC", None, "ASO"]
}