 PDF To Markdown Converter
Debug View
Result View
Index Guideline
VERSION 1.

11 J A N U A R Y 2 0 2 4
Solactive Systematic Trend Alpha Replicator Excess Return Index

TAB LE O F CO N TE N TS
4
Version 1.0 – 02 November 2023
INTRODUCTION
This document (the “GUIDELINE”) is to be used as a guideline with regard to the composition, calculation
and maintenance of the Solactive Systematic Trend Alpha Replicator Excess Return Index (the “INDEX”).
Any amendments to the rules made to the Guideline are approved by the INDEX COMMITTEE specified in
Section 4. 5. The INDEX is owned, calculated, maintained, administered and published by Solactive AG
(“SOLACTIVE”) assuming the role as index owner (the “INDEX OWNER”) and index administrator (the “INDEX
ADMINISTRATOR”) under the Regulation (EU) 2016/1011 (the “BENCHMARK REGULATION” or “BMR”). The name
“Solactive” is trademarked.

The INDEX OWNER owns the copyright and all other intellectual property rights in the INDEX. Any use of
these intellectual property rights may only be made with the prior written consent of the INDEX OWNER.

In no event shall the INDEX OWNER be liable (whether directly or indirectly, in contract, tort or otherwise)
for any loss incurred by any person that arises out of or in connection with the INDEX, including in relation
to the performance by the INDEX OWNER of any part of its role in respect of the INDEX, save in the case of
gross negligence, fraud or wilful default. In no event, shall the INDEX OWNER have any liability to any
persons for any direct, indirect, special, punitive or consequential damages (including lost profits) even if
notified of the possibility of such damages.

With respect to any products linked to the INDEX, the INDEX OWNER expressly disclaims all liability for
regulatory, juridical or reputational consequences suffered by any party in any transaction connected
with the INDEX.

The INDEX OWNER does not guarantee the quality, accuracy and/or completeness of any data or
information provided by the INDEX CONSULTANT (as defined below) for the purposes of calculating the INDEX
(the “relevant data”) and the INDEX OWNER shall have no liability for any errors, omissions, delays or
interruptions relating to the relevant data on the part of the INDEX CONSULTANT or otherwise.

The INDEX will be governed by the INDEX ADMINISTRATOR. The INDEX ADMINISTRATOR controls the creation and
operation of the INDEX, including (but not limited to) all stages and processes involved in the production,
calculation, maintenance, administration and dissemination of the INDEX. Notwithstanding that the INDEX
relies on information from third party sources, the INDEX ADMINISTRATOR has primary responsibility for all
aspects of the index administration and determination process.

The GUIDELINE and the policies and methodology documents referenced herein contain the
underlying principles and rules regarding the structure and operation of the INDEX. The INDEX
ADMINISTRATOR does not offer any explicit or tacit guarantee or assurance, neither pertaining to the
results from the use of the INDEX nor the level of the INDEX at any certain point in time nor in any
other respect. The INDEX ADMINISTRATOR strives to the best of its ability to ensure the correctness of

5
Version 1.0 – 02 November 2023
the calculation. There is no obligation for the INDEX ADMINISTRATOR – irrespective of possible
obligations to issuers – to advise third parties, including investors and/or financial intermediaries,
of any errors in the INDEX. The publication of the INDEX by the INDEX ADMINISTRATOR does not
constitute a recommendation for capital investment and does not contain any assurance or
opinion of the INDEX ADMINISTRATOR regarding a possible investment in a financial instrument based
on this INDEX.

In calculating the INDEX, the INDEX ADMINISTRATOR will obtain and use the relevant data. The INDEX OWNER (or
any of its respective affiliates or subsidiaries or any of its respective directors, officers, employees,
representatives, delegates or agents) will not independently verify the relevant data, does not guarantee
the quality, accuracy and/or the completeness of the relevant data and consequently the INDEX OWNER (or
any of its respective affiliates or subsidiaries or any of its respective directors, officers, employees,
representatives, delegates or agents) does not guarantee the quality, accuracy and/or completeness of
such relevant data (and, therefore, the INDEX). The INDEX OWNER (or any of its respective affiliates or
subsidiaries or any of its respective directors, officers, employees, representatives, delegates or agents)
shall not be liable (whether in contract, tort or otherwise) to any person for any inaccuracy, omission,
mistake or error in the relevant data and the INDEX OWNER is not under any obligation to advise any person
of any inaccuracy, omission, mistake or error it becomes aware of in respect of the relevant data.

The INDEX OWNER (or any of its respective affiliates or subsidiaries or any of its respective directors, officers,
employees, representatives, delegates or agents) does not make, and each of them disclaims, any express
or implied representations or warranties with respect to the relevant data, including, by way of example of
merchantability or fitness for a particular purpose.

Ai For Alpha ( the “INDEX CONSULTANT”) is a leading provider of AI-powered market signals and investment
models to investment professionals.

As a model provider, the INDEX CONSULTANT’s role is limited to providing data, computation and model
weights. Data, computation and model weights communicated by the INDEX CONSULTANT, are confidential
and may not be recopied, reproduced or otherwise redistributed without prior consent. These data,
computation and model weights are based on certain modeling assumptions that may differ significantly
from the actual costs, real execution and performance. These calculations should not be construed as
providing legal, regulatory, tax, financial, or investment advice.

The INDEX CONSULTANT does not guarantee the accuracy or completeness of the computed target weights
and accepts no liability for any consequential losses arising from the use of this information. Any data on
past performance herein is no indication as to future performance.

The text uses defined terms which are formatted with “SMALL CAPS”. Such Terms shall have the meaning
assigned to them as specified in Section 5 (Definitions).

6
Version 1.0 – 02 November 2023
1. Index Specifications
1.1. Scope of the Index
Category Description
Asset Class Multi asset
Strategy The INDEX is a USD denominated index.
The INDEX is comprised of a static basket of 13 constituents (the “INDEX COMPONENTS”),
of which 4 are exchange traded funds (each a “ETF”), 2 are FX futures contracts, 4 are
equity futures contracts and 3 are bond futures contracts.
The INDEX COMPONENTS are combined to form a base index (the “BASE INDEX”). The INDEX
COMPONENTS performance is currency converted, where applicable, so that the BASE
INDEX is denominated in USD.
The weights allocated to the INDEX COMPONENTS are determined based on target
weights provided on each CALCULATION DAY by INDEX CONSULTANT (such target weights
being the “TARGET WEIGHTS”).
Rebalancing
Frequency
Daily
Adjustment
Factor
The INDEX takes exposure to the BASE INDEX, and calculated as an excess return index,
subject to an adjusted return factor of 0.4%.
1.2. About Ai For Alpha
The INDEX CONSULTANT is a technology provider that uses artificial intelligence to help investment
professionals understand the key drivers of financial markets and make informed investment decisions. It
uses various machine learning techniques to identify meaningful relationships between financial factors
and investment strategies. The goal is to leverage a variety of market information and help investors
improve their investment process.

In particular, the INDEX CONSULTANT has developed a proprietary machine learning model to decode the
positioning of asset classes based on their historical valuations.

The model output provides daily weights with respect to each INDEX COMPONENT.

7
Version 1.0 – 02 November 2023
1.3. Identifiers and Publication
The INDEX is published under the following identifiers:

Name ISIN
Index
Currency
Type RIC BBG ticker
Solactive Systematic
Trend Alpha Replicator
Excess Return Index
DE000SL0KU22 USD AR* .SOLSTAE SOLSTAE Index
*AR means that the INDEX is calculated as Adjusted Return.

The INDEX is published on the website of the INDEX ADMINISTRATOR (www.solactive.com) (or any successor
source thereto) and is, in addition, available via the price marketing services of Boerse Stuttgart GmbH and
may be distributed to all its affiliated vendors. Each vendor decides on an individual basis as to whether it
will distribute or display the INDEX via its information systems.

Any publication in relation to the INDEX (e.g., notices, amendments to the GUIDELINE) will be available at the
website of the INDEX ADMINISTRATOR: https://www.solactive.com/news/announcements/.

1.4. Initial Level of the Index
The initial level of the INDEX on the 20 06 /0 7 / 13 (the “START DATE”), is 100. Historical values from the
2024 / 01 / 03 (the “LIVE DATE”), will be recorded in accordance with Article 8 of the BMR. Levels of the INDEX
published for a period prior to the LIVE DATE have been back-tested.

1.5. Prices and calculation frequency
The intraday level of the INDEX is calculated by the INDEX ADMINISTRATOR on each CALCULATION DAY during the
CALCULATION WINDOW based on the TRADING PRICES on the EXCHANGES on which the INDEX COMPONENTS
comprised in the BASE INDEX are listed. TRADING PRICES of INDEX COMPONENTS not listed in the INDEX CURRENCY
will be converted by the INDEX ADMINISTRATOR using the then current Intercontinental Exchange (ICE) spot
foreign exchange rate. If there is no current TRADING PRICE for an INDEX COMPONENT in respect of a CALCULATION
DAY, the later of: (i) the most recent CLOSING PRICE in respect of such INDEX COMPONENT; or (ii) the last available
TRADING PRICE in respect of such INDEX COMPONENT for the preceding TRADING DAY is used in the calculation.

In addition to the intraday calculation described above, the INDEX ADMINISTRATOR will also calculate a closing
level of the INDEX for each CALCULATION DAY. This closing level is based on the CLOSING PRICES of the INDEX
COMPONENTS on the respective EXCHANGES on which the INDEX COMPONENTS are listed. If there is no CLOSING
PRICE for an INDEX COMPONENT in respect of a CALCULATION DAY, the most recent TRADING PRICE in respect of
such INDEX COMPONENT is used for such closing level calculation.

8
Version 1.0 – 02 November 2023
The FUTURES LEVEL of INDEX COMPONENTS not listed in the INDEX CURRENCY are converted using the 04:00 p.m.
London time rates provided by WM/ Refinitiv (the “WM/REFINITIV RATE”). If there is no 04:00 p.m. London time
WM/REFINITIV RATE for the relevant CALCULATION DAY, the last available 04:00 p.m. London time WM/REFINITIV
RATE will be used for the closing level calculation.

If data cannot be provided to the pricing services of Boerse Stuttgart GmbH, the INDEX cannot be distributed
and is only available via the website of the INDEX ADMINISTRATOR.

For more information regarding the treatment of non-available current TRADING PRICES, CLOSING PRICES
PRICES, and WM/REFINITIV RATESin the index calculation, please refer to our Solactive Equity Index
Methodology, which is available on the SOLACTIVE website: https://www.solactive.com/documents/equity-
index-methodology/.
Any corporate action event applicable to any of the INDEX COMPONENTS will be treated in accordance with
Section 3.8 of this Guideline and Section 2.1 of the Solactive Equity Index Methodology which is
incorporated by reference and available on the SOLACTIVE website (or any successor source thereto):
https://www.solactive.com/documents/equity-index-methodology/.

1.6. Licensing..................................................................................................................................................................................................................
Licenses to use the INDEX as the underlying value for financial instruments, investment funds and
financial contracts may be issued to stock exchanges, banks, financial services providers and investment
houses and any such licenses may only be issued by the INDEX OWNER.

2. Index Selection
No selection of INDEX COMPONENTS takes place.

2.1. Index Universe Requirements...........................................................................................................................................................................
Not applicable as no ordinary rebalance takes place.

2.2. Index Components
The BASE INDEX is based on the basket containing the 13 components (the “BASKET”) set out in the following
table (“TABLE 1 IDENTIFIER”). Please refer to Appendix 1 for the detailed methodology relating to rolling
futures.

Name RIC Return Type Asset Type Exchange
E-mini S&P 500 Futures 0#ES: ER
Equity
Futures
CME
9
Version 1.0 – 02 November 2023
E-mini Nasdaq-100 Futures 0#NQ:
ER Equity
Futures
CME
10y Treasury Note Futures 0#TY:
ER Bond
Futures
CBOT
2y Treasury Note Futures 0#TU:
ER Bond
Futures
CBOT
EUR/USD Futures 0#URO: ER^ FX Futures CME^
JPY/USD Futures 0#JY: ER^ FX Futures CME^
Japan 225 Futures 0#NIY:
ER Equity
Futures
CME
Euro 50 Futures 0#STXE:
ER Equity
Futures
EUREX
Bund Futures 0#FGBL:
ER Bond
Futures
EUREX
iShares MSCI Emerging Markets ETF EEM.P ER^ ETF NYSE^
SPDR Gold Shares GLD.P ER^ ETF NYSE^
Energy Select Sector SPDR Fund XLE.P ER^ ETF NYSE^
SPDR S&P Metals and Mining ETF XME.P ER^ ETF NYSE^
Table 1 Identifier

2.3. Weighting of the Index Components
The weights of the INDEX COMPONENTS will be determined and provided by the INDEX CONSULTANT to the INDEX
ADMINISTRATOR on each CALCULATION DAY immediately preceding CALCULATION DAY t.

10
Version 1.0 – 02 November 2023
3. Calculation of the Index
3.1. Index formula
The level of the INDEX is (i) an adjusted return index subject to an adjusted return factor of 0.4% p.a. (ii) and
calculated based on the level of the BASE INDEX, which is calculated as the weighted average performance
of the INDEX COMPONENTS.

The level of the INDEX as of START DATE is set to 100:

𝐼𝑛𝑑𝑒𝑥𝑆𝑡𝑎𝑟𝑡 𝐷𝑎𝑡𝑒= 100
The level of the INDEX as of any CALCULATION DAY t from (but excluding) the START DATE is calculated
according to the following formula:

𝐼𝑛𝑑𝑒𝑥𝑡=max [ 0 ,𝐼𝑛𝑑𝑒𝑥𝑡− 1 ∗(
𝐵𝑡
𝐵𝑡− 1
−𝐴𝑅𝐹×
𝐷𝐶𝐹𝑡,𝑡− 1
365
−𝑇𝑇𝐶𝑡− 𝑇𝑅𝐶𝑡)]
Where:

𝐴𝑅𝐹: The adjusted return factor of 0.4%.
𝐵𝑡: The level of the BASE INDEX as of CALCULATION DAY t.
𝐵𝑡− 1 : The level of the BASE INDEX as of the CALCULATION DAY immediately preceding CALCULATION DAY t.
𝐷𝐶𝐹𝑡,𝑡− 1 : The number of calendar days between CALCULATION DAY 𝑡 (including) and CALCULATION DAY t- 1
(excluding).
𝐼𝑛𝑑𝑒𝑥𝑡− 1 : The level of the INDEX as of the CALCULATION DAY immediately preceding CALCULATION DAY t.
𝑇𝑅𝐶𝑡: is the TOTAL REPLICATION COST and is calculated as described in Section 3.5.
𝑇𝑇𝐶𝑡: is the TOTAL TRANSACTION COST and is calculated as described in Section 3. 4.

3.2. Base Index Calculation
The respective weightings of each INDEX COMPONENT comprised in the BASE INDEX are adjusted on each
CALCULATION DAY using the TARGET WEIGHTS provided by the INDEX CONSULTANT as of CALCULATION DAY 𝑡− 1 ,
effective on CALCULATION DAY t.
The level of the BASE INDEX as of any CALCULATION DAY t is calculated according to the following formula:

𝐵𝑡=𝐵𝑡− 1 ×( 1 + ∑𝑤𝑖,𝑡×(
𝐼𝐶𝑖,𝑡
𝐼𝐶𝑖,𝑡− 1
− 1 )
13
𝑖= 1
)
Where:
The BASE INDEX as of START DATE is set to 100:

11
Version 1.0 – 02 November 2023
𝐵𝑆𝑡𝑎𝑟𝑡 𝐷𝑎𝑡𝑒= 100
𝐵𝑡− 1 : The level of the BASE INDEX as of the CALCULATION DAY immediately preceding CALCULATION DAY 𝑡.
𝐼𝐶𝑖,𝑡: LEVEL of INDEX COMPONENT 𝑖 as of CALCULATION DAY 𝑡.
𝐼𝐶𝑖,𝑡− 1 : LEVEL of INDEX COMPONENT 𝑖 as of the CALCULATION DAY immediately preceding CALCULATION DAY 𝑡.
𝑤𝑖,𝑡: the TARGET WEIGHTS of INDEX COMPONENT 𝑖 for Calculation Day 𝑡, as provided by the INDEX CONSULTANT.
LEVEL means, in respect of a CALCULATION DAY and an INDEX COMPONENT with an “Asset Type” (as specified in
Table 1 Identifier above) of:

(i) ETF, the amount determined in accordance with Section 3.3 below; and
(ii) Bond Futures, FX Futures or Equity Futures, the Rolling Future Level determined in accordance
with Section 2.1 of Appendix 1 (converted as necessary into the Index in accordance with Section
2.1.5 of Appendix 1),
in each case, in respect of such CALCULATION DAY.
If any CALCULATION DAY 𝑡 is not a TRADING DAY for any INDEX COMPONENT 𝑖, 𝐼𝐶𝑖,𝑡− 1 is considered for
calculating 𝐵𝑡. Additionally, if the TARGET WEIGHTS 𝑤𝑖 aren’t provided by the INDEX CONSULTANT for any INDEX
COMPONENT 𝑖 for any CALCULATION DAY 𝑡, the INDEX is considered to be on holiday on such CALCULATION DAY t
and no INDEX value will be published on CALCULATION DAY 𝑡.

E T F L E V E L C A L C U L A T I O N
The LEVEL as of CALCULATION DAY 𝑡 for each INDEX COMPONENT 𝑖 (𝐼𝐶𝑖,𝑡) that is an ETF is calculated (i) by
reinvesting the dividends, and (ii) calculating excess return over the SECURED OVERNIGHT FINANCING RATE.
The excess return level is set at 100 as of the START DATE for each INDEX COMPONENT 𝑖 that is an ETF as
listed in Section 2.2.
𝐸𝑇𝐹𝐿𝑒𝑣𝑒𝑙𝑆𝑡𝑎𝑟𝑡 𝐷𝑎𝑡𝑒= 100

The excess return level of each INDEX COMPONENT 𝑖 that is an ETF is as listed in Section 2.2 as of any
CALCULATION DAY t from (but excluding) the START DATE is calculated according to the following formula:

𝐸𝑇𝐹𝐿𝑒𝑣𝑒𝑙𝑡=𝐸𝑇𝐹𝐿𝑒𝑣𝑒𝑙𝑡− 1 ×(
𝐸𝑇𝐹_𝐶𝑙𝑜𝑠𝑒𝑡+𝑑𝑖𝑣𝑡
𝐸𝑇𝐹_𝐶𝑙𝑜𝑠𝑒𝑡− 1
−(𝑟𝑎𝑡𝑒𝑡− 2 ×
𝐷𝐶𝐹𝑡,𝑡− 1
365
))
Where:
𝐸𝑇𝐹𝐿𝑒𝑣𝑒𝑙𝑡− 1 : The excess return level of INDEX COMPONENT 𝑖 that is an ETF as of the CALCULATION DAY
immediately preceding CALCULATION DAY t.
𝐸𝑇𝐹𝐶𝑙𝑜𝑠𝑒𝑡: The CLOSING PRICE of INDEX COMPONENT 𝑖 that is an ETF as of CALCULATION DAY t.
𝐸𝐹𝑇𝐶𝑙𝑜𝑠𝑒𝑡− 1 : The CLOSING PRICE of INDEX COMPONENT 𝑖 that is an ETF as of the CALCULATION DAY
immediately preceding CALCULATION DAY t.
𝑟𝑎𝑡𝑒𝑡 is the interest rate determined as described below:

12
Version 1.0 – 02 November 2023
if CALCULATION DAY t is after or equal to the RATE SWITCH DATE, then it is equal to the SECURED
OVERNIGHT FINANCING RATE
if CALCULATION DAY t is prior to the RATE SWITCH DATE, then it is equal to the 3 - MONTH USD-LIBOR
minus 0.26161%.
𝐷𝐶𝐹𝑡,𝑡− 1 : The number of calendar days between CALCULATION DAY t (including) and CALCULATION DAY t- 1
(excluding).
𝑑𝑖𝑣𝑡: The CASH DIVIDENDS of INDEX COMPONENT 𝑖 that is an ETF with an ex-date corresponding to CALCULATION
DAY tas determined by the INDEX ADMINISTRATOR. Adjustments to the INDEX to account for CASH DIVIDENDS
will be made in compliance with the Equity Index Methodology, which is available on the SOLACTIVE
website: https://www.solactive.com/documents/equityindex-methodology, and as described in Section
3.8.

3.4. Total Transaction Cost Calculation
For the purposes of the index formula at Section 3.1, the fixed transaction cost, “𝑓𝑡𝑐”, is set at 0.02%.
For CALCULATION DAY 𝑡+ 1 , where 𝑡 is the START DATE, the total transaction cost (“TOTAL TRANSACTION
COST” or “𝑇𝑇𝐶”) is calculated as follows:

𝑇𝑇𝐶𝑡=∑𝑓𝑡𝑐×𝐴𝐵𝑆(𝑤𝑖,𝑡)
13
𝑖= 1
For any other CALCULATION DAY 𝑡, total transaction cost is calculated as the aggregate of the product of
the fixed transaction cost 𝑓𝑡𝑐 and the absolute difference of the weight/allocation to INDEX COMPONENT 𝑖
between CALCULATION DAY 𝑡 and CALCULATION DAY 𝑡− 1.

𝑇𝑇𝐶𝑡=∑𝑓𝑡𝑐 ×𝐴𝐵𝑆(𝑤𝑖,𝑡− 𝑤𝑖,𝑡− 1
13
𝑖= 1
)
Where:
𝐴𝐵𝑆(𝑤𝑖,𝑡) : The absolute value of Target Weight of INDEX COMPONENT 𝑖 for Calculation Day 𝑡.

3.5. Total Replication Cost Calculation
For the purposes of the index formula at Section 3.1, the total replication cost (“TOTAL REPLICATION COST” or
“𝑇𝑅𝐶”) as of CALCULATION DAY 𝑡, is defined as follows:

𝑇𝑅𝐶𝑡=∑𝑅𝐶 ×𝐴𝐵𝑆(𝑤𝑖,𝑡
13
𝑖= 1
)×
𝐷𝐶𝐹𝑡,𝑡− 1
365
Where:
𝐴𝐵𝑆(𝑤𝑖,𝑡) : The absolute value of TARGET WEIGHT of INDEX COMPONENT 𝑖 for Calculation Day 𝑡
𝑅𝐶: is the replication cost, set at 0.15% for each INDEX COMPONENT 𝑖 with an Asset Type (as specified in
Tabe 1 Identifier above) of “Bond Futures”, “FX Futures” or “Equity Futures”, else it is set at 0.0% for each
INDEX COMPONENT 𝑖 with an Asset Type (as specified in Tabe 1 Identifier above) of type “ETF”.

13
Version 1.0 – 02 November 2023
𝐷𝐶𝐹𝑡,𝑡− 1 : The number of calendar days between CALCULATION DAY t (including) and CALCULATION DAY t- 1
(excluding).

3.6. Accuracy
The level of the INDEX will be rounded to 2 decimal places (0.005 being rounded upwards).

3.7. Adjustments
Under certain circumstances, an adjustment of the INDEX may be necessary. Such adjustment has to be
made if a corporate action (as specified in Section 3.8 below) in relation of an INDEX COMPONENT occurs. Such
adjustment may have to be done in relation to an INDEX COMPONENT and/or may also affect the number of
INDEX COMPONENTS and/or the weighting of certain INDEX COMPONENTS and will be made in compliance with
the SOLACTIVE Equity Index Methodology.

THE INDEX ADMINISTRATOR will announce the INDEX adjustment giving a notice period of at least two TRADING
DAYS (with respect to the affected INDEX COMPONENT) on the SOLACTIVE website under the Section
“Announcements”, which is available at https://www.solactive.com/news/announcements/ (or any
successor source thereto). The INDEX adjustments will be implemented on the effective day specified in the
respective announcement.

3.8. Corporate actions
As part of the INDEX maintenance the INDEX ADMINISTRATOR will consider various events – also referred to as
corporate actions – which result in an adjustment to the INDEX between two consecutive regular
CALCULATION DAYS. Such events have a material impact on the price, weighting or overall integrity of an INDEX
COMPONENT. Therefore, they need to be accounted for in the calculation of the INDEX. Corporate actions will
be implemented from the cum-day to the ex-day of the corporate action, so that the adjustment to the
INDEX coincides with the occurrence of the price effect of the respective corporate action.

Adjustments to the INDEX to account for corporate actions will be made by the INDEX ADMINISTRATOR in
compliance with the Equity Index Methodology. This document contains for each corporate action a brief
definition and specifies the relevant adjustment to the INDEX variables.

While the INDEX ADMINISTRATOR aims at creating and maintaining its methodology for the treatment of
corporate actions as generic and transparent as possible and in line with regulatory requirements, the
INDEX ADMINISTRATOR retains the right in accordance with the Equity Index Methodology to deviate from
these standard procedures in case of any unusual or complex corporate action or if such a deviation is made
to preserve the comparability and representativeness of the INDEX over time.

14
Version 1.0 – 02 November 2023
The INDEX ADMINISTRATOR considers the following, but not conclusive, list of corporate actions as relevant
for the BASKET maintenance:

Cash distributions (e.g., payment of a dividend)
Stock distributions (e.g., payment of a dividend in form of additional shares)
Stock distributions of another company (e.g. payment of a dividend in form of additional shares of
another company (e.g. of a subsidiary))
Share splits (company ́s present shares are divided and therefore multiplied by a given factor)
Reverse splits (company ́s present shares are effectively merged)
Capital increases (such as issuing additional shares)
Share repurchases (a company offer its shareholders the option to sell their shares to a fixed price)
Spin-offs (the company splits its business activities into two or more entities and distributes new
equity shares in the created entities to the shareholders of the former entity)
Mergers & Acquisitions (transaction in which the ownership of a company (or other business
organizations) are transferred or consolidated with other entities, e.g. fusion of two or more
separate companies into one entity)
Delistings (company ́s shares are no longer publicly traded at a stock exchange)
Nationalization of a company (effective control of a legal entity is taken over by a state)
Insolvency
3.9. Recalculation
The INDEX ADMINISTRATOR makes the greatest possible efforts to accurately calculate and maintain its INDEX.
However, errors in the index determination process may occur from time to time for a variety of reasons
(internal or external) and therefore, cannot be completely ruled out in respect of the INDEX. The INDEX
ADMINISTRATOR endeavors to correct all errors that have been identified within a reasonable period of time.
The understanding of “a reasonable period of time” as well as the general measures to be taken are
generally depending on the underlying and is specified in the Solactive Correction Policy, which is
incorporated by reference and available on the INDEX ADMINISTRATOR website (or any successor source
thereto): http://www.solactive.com/documents/correction-policy.

15
Version 1.0 – 02 November 2023
3.10. Market Disruption
In periods of market stress the INDEX ADMINISTRATOR shall calculate the INDEX following predefined and
exhaustive arrangements as described in the Solactive Disruption Policy, which is incorporated by
reference and available on the INDEX ADMINISTRATOR website (or any successor source thereto):
https://www.solactive.com/documents/disruption-policy/. Market stress can arise due to a variety of
reasons, but generally results in inaccurate or delayed prices for one or more INDEX COMPONENTS. The
determination of the Index may be limited or impaired at times of illiquid or fragmented markets and
market stress.

4. Derived Indices
The following indices are derived from the INDEX:

Name ISIN
Index
Currency
Type RIC BBG ticker
Solactive Systematic
Trend Replicator GBP
hedged
DE000SL0L649 GBP AR*. SOLSTAG SOLSTAG Index
The DERIVED INDEX is a currency hedged version of the INDEX and is calculated according to the following:

𝐼𝑛𝑑𝑒𝑥𝑡𝐹𝑋=𝐼𝑛𝑑𝑒𝑥𝑡𝐹𝑋− 1 ∗( 1 +(
𝐼𝑛𝑑𝑒𝑥𝑡
𝐼𝑛𝑑𝑒𝑥𝑡− 1
− 1 )∗
𝐹𝑋𝑡
𝐹𝑋𝑡− 1
)
Where:

𝐼𝑛𝑑𝑒𝑥𝑡𝐹𝑋: The DERIVED INDEX (hedged to the currency indicated by column “Index Currency”) as of

CALCULATION DAY t

𝐼𝑛𝑑𝑒𝑥𝑡𝐹𝑋− 1 : The DERIVED INDEX (hedged to the currency indicated by column “Index Currency”) as of the

CALCULATION DAY immediately preceding CALCULATION DAY t

𝐼𝑛𝑑𝑒𝑥𝑡: The level of the INDEX as of the CALCULATION DAY t

𝐼𝑛𝑑𝑒𝑥𝑡− 1 :The level of the INDEX as of the CALCULATION DAY immediately preceding CALCULATION DAY t.

𝐹𝑋𝑡: The exchange rate to convert one unit of USD to the hedged currency as of CALCULATION DAY t

𝐹𝑋𝑡− 1 : The exchange rate to convert one unit of USD to the hedged currency as of immediately

preceding CALCULATION DAY t

16
Version 1.0 – 02 November 2023
5 Miscellaneous
5 .1. Discretion
Any discretion which may need to be exercised in relation to the determination of the INDEX shall be limited
to (i) exercising routine judgement (in the expert view of the INDEX ADMINISTRATOR) in the administration of
the INDEX rules, provided, however, that such routine judgment does not include deviations or alterations
to the Index rules that are designed to improve the financial performance of the INDEX, (ii) correcting errors
in the implementation of the rules or calculations made pursuant to the INDEX rules, or (iii) making an
adjustment to respond to an unanticipated event outside of INDEX ADMINISTRATOR’S control, such as a stock
split, merger, listing or delisting, nationalization, or insolvency, a disruption in the financial markets for
specific assets or in a particular jurisdiction, regulatory compliance requirement, force majeure, or any
other unanticipated event of similar magnitude and significance, in each case subject to sections 3. 9 to

10 hereof.
5 .2. Methodology Review
The methodology of the INDEX is subject to regular review, at least annually. If a change in methodology
has been identified as being needed as a result of such review (e.g. if the underlying market or economic
reality has changed since the launch of the INDEX or if the present methodology is based on obsolete
assumptions and factors and no longer reflects the reality as accurately, reliably and appropriately as
before), such change will be made in accordance with the INDEX ADMINISTRATOR Methodology Policy, which
is incorporated by reference and available on the INDEX ADMINISTRATOR website (or any successor source
thereto): https://www.solactive.com/documents/methodology-policy/.

Any such change in the methodology will be announced on the INDEX ADMINISTRATOR website under the
Section “Announcement”, which is available at https://www.solactive.com/news/announcements/ (or any
successor source thereto).

5 .3. Changes in calculation method
The application by the INDEX ADMINISTRATOR of the method described in this document is final and binding.
The INDEX ADMINISTRATOR shall apply the method described above for the composition and calculation of the
INDEX. However, it cannot be excluded that the market environment, supervisory, legal and financial or tax
reasons may require changes to be made to this method. The INDEX ADMINISTRATOR may also make changes
to the terms and conditions of the INDEX and the method applied to calculate the INDEX that it deems to be
necessary and desirable in order to prevent obvious or demonstrable error or to remedy, correct or
supplement incorrect terms and conditions. Any such changes or modifications made in respect of the
INDEX in accordance with this Section 4.3 shall be made by the INDEX ADMINISTRATOR in accordance with its
policies. The INDEX ADMINISTRATOR is not obliged to provide information on any such modifications or

17
Version 1.0 – 02 November 2023
changes. Despite any modifications and changes, the INDEX ADMINISTRATOR will take the appropriate steps to
ensure a calculation method is applied that is consistent with the method described above.

5 .4. Termination
The INDEX ADMINISTRATOR shall make the greatest possible efforts to ensure the resilience and continued
integrity of the INDEX over time. Where necessary, the INDEX ADMINISTRATOR shall follow a clearly defined and
transparent procedure to adapt the INDEX methodology to account for changing underlying markets (see
Section 5 .2 “Methodology Review”) in order to maintain continued reliability and comparability of the INDEX.
Nevertheless, if no other options are available the orderly cessation of the INDEX may be indicated. This is
usually the case when the underlying market or economic reality, which the INDEX is set to measure or to
reflect, changes substantially and in a way not foreseeable at the time of inception of the INDEX, the INDEX
methodology can no longer be applied coherently or the INDEX is no longer used as the underlying value for
financial instruments, investment funds and financial contracts.

The INDEX ADMINISTRATOR has established and maintains clear guidelines on how to identify situations in
which the cessation of an index is unavoidable, how stakeholders are to be informed and consulted and the
procedures to be followed for a termination or the transition to an alternative index. Details are specified
in the INDEX ADMINISTRATOR Termination Policy, which is incorporated by reference and available on the INDEX
ADMINISTRATOR website (or any successor source thereto):
https://www.solactive.com/documents/termination-policy/.

5 .5. Oversight
An index committee composed of staff from the INDEX ADMINISTRATOR and its subsidiaries (the “INDEX
COMMITTEE”) is responsible for decisions regarding any amendments to the rules of the Index. Any such
amendment, which may result in an amendment of the Guideline, must be submitted to the INDEX
COMMITTEE for prior approval and will be made in compliance with the Methodology Policy, which is available
on the SOLACTIVE website: https://www.solactive.com/documents/methodology-policy/.

6. DEFINIT IONS
This section contains defined terms used throughout this GUIDELINE document, other than Appendix 1
which includes additional definitions which apply to that Appendix 1.

“ 3 - MONTH USD-LIBOR” means RIC: USD3MFSR=.
“BASE INDEX” shall have the meaning as defined in Section 1. 1.
“BASKET” shall have the meaning as defined in Section 2.2.
“BENCHMARK REGULATION” shall have the meaning as defined in Section “Introduction”.
“BMR” shall have the meaning as defined in Section “Introduction”.

18
Version 1.0 – 02 November 2023
“CALCULATION DAY” is a day on which both the New York Stock Exchange (XNYS) and the Chicago Board of
Trade (XCBT) are open for general business.
“CALCULATION WINDOW” means, in respect of any Calculation Day, the period commencing on (and
including) 9:00 CET and ending on (and including) 22:50 CET, in each case, on such Calculation Day.
“CLOSE OF BUSINESS” is the calculation time of the closing level of the INDEX as outlined in Section 1. 5.
The “CLOSING PRICE” in respect of an INDEX COMPONENT and a TRADING DAY is a security’s final regular-hours
TRADING PRICE published by the EXCHANGE and determined in accordance with the EXCHANGE regulations. If
the EXCHANGE has no or has not published a CLOSING PRICE in accordance with the EXCHANGE rules for an
INDEX COMPONENT, the last TRADING PRICE will be used.
“CASH DIVIDENDS” refers to both ordinary and special dividend payments.
“DERIVED INDEX” refers to all indices listed in Section 4.
“EXCHANGE” is with respect to the INDEX and every INDEX COMPONENT, the respective exchange where the
INDEX COMPONENT has its listing as specified in the column “Exchange” in the Table 1, Section 2.2.
“FUTURES LEVEL” shall have the meaning as defined in Appendix 1, Section 2.1.4.
“GUIDELINE” shall have the meaning as defined in Section “Introduction”.
“INDEX” shall have the meaning as defined in Section “Introduction”.
“INDEX ADMINISTRATOR” shall have the meaning as defined in Section “Introduction”.
“INDEX COMMITTEE” shall have the meaning as defined in Section 5 .5.
“INDEX COMPONENT” shall have the meaning as defined in Section 1.1.
“INDEX CONSULTANT” shall have the meaning as defined in Section “Introduction”.
“INDEX CURRENCY” is the currency specified in the column “Index Currency” in the table in Section 1. 3.
“INDEX OWNER” shall have the meaning as defined in Section “Introduction”.
“INDEX UNIVERSE” is the sum of all financial instruments which fulfill the INDEX UNIVERSE REQUIREMENTS.
“INDEX UNIVERSE REQUIREMENTS” shall have the meaning as defined in Section 2.1.
“LIVE DATE” shall have the meaning as defined in Section 1. 4.
“RATE SWITCH DATE” shall be 2020/12/31.
“SECURED OVERNIGHT FINANCING RATE” means RIC: USDSOFR=.
“SOLACTIVE” shall have the meaning as defined in Section “Introduction”.
“START DATE” shall have the meaning as defined in Section 1. 4.
“TABLE 1 IDENTIFIER” has the meaning given to it in Section 2.2.
“TOTAL REPLICATION COST” has the meaning given to it in Section 3.5.
“TOTAL TRANSACTION COST” has the meaning given to it in Section 3.4.
“TRADING DAY” is with respect to an INDEX COMPONENT, a day on which the relevant EXCHANGE is open for
trading (or a day that would have been such a day if a market disruption had not occurred), excluding days
on which trading may be ceased prior to the scheduled EXCHANGE closing time and days on which the
EXCHANGE is open for a scheduled shortened period. The INDEX ADMINISTRATOR is ultimately responsible for
determining whether a certain day is a TRADING DAY.
The “TRADING PRICE” in respect of an INDEX COMPONENT is the most recent published price at which the
INDEX COMPONENT was traded on the respective EXCHANGE.
“WM / REFINITIV RATE” shall have the meaning as defined in Section 1. 5.

  -
Version 1.0 – 02 November
Index Specifications
1.1. Scope of the Index
1.2. About Ai For Alpha
1.3. Identifiers and Publication
1.4. Initial Level of the Index
1.5. Prices and calculation frequency
1.6. Licensing..................................................................................................................................................................................................................
Index Selection
2.1. Index Universe Requirements...........................................................................................................................................................................
2.2. Index Components
2.3. Weighting of the Index Components
Calculation of the Index
3.1. Index formula
3.2. Base Index Calculation
3.4. Total Transaction Cost Calculation
3.5. Total Replication Cost Calculation
3.6. Accuracy
3.7. Adjustments
3.8. Corporate actions
3.9. Recalculation
3.10. Market Disruption
Derived Indices
5 Miscellaneous
5 .1. Discretion
5 .2. Methodology Review
5 .3. Changes in calculation method
5 .4. Termination
5 .5. Oversight
6 Definitions
Appendix
Rolling Futures
1.1. Futures Components.........................................................................................................................................................................................
-
Version 1.0 – 02 November
1.2. Selection of the Futures Contracts
1.3. Weighting of the Contracts
Calculation of the Rolling Future Level
2.1. Formula
2.1.1. Future Return
2.1. 2 Future Level
2.1. 3 FX Conversion
Appendix 2 – Tables
Appendix 3 – Definitions
Contact
Version 1.0 – 02 November
20
Version 1.0 – 02 November 2023
Appendix
1. Rolling Futures
1.1. Futures Components.........................................................................................................................................................................................
For the purposes of this Appendix 1, any INDEX COMPONENT that comprises FUTURES CONTRACTS belonging to a
FUTURES CHAIN shall be deemed to be “FUTURES COMPONENTS”.

1.2. Selection of the Futures Contracts
Table 4 (Expiration Month of the Active Contract) of Appendix 2, defines the ACTIVE CONTRACT’S expiration
month per calendar month of CALCULATION DAY 𝑡, which is the FUTURES CONTRACT in which the FUTURES
COMPONENTS is invested on the first day of the calendar month of t as per Table 4 (Expiration Month of the
Active Contract) of Appendix 2.

The “NEXT ACTIVE CONTRACT” is the future contract in which the future component rolls into between the
ROLL START and the ROLL END.

Table 5 (Expiration Month of the Next Active Contract) of Appendix 2 defines the NEXTACTIVE CONTRACT’S
expiration month per calendar month of CALCULATION DAY 𝑡

A plus sign “+” (if applicable) means that expiration year with respect to an NEXT ACTIVE CONTRACT is the
year following the current year of CALCULATION DAY t.

E.g: Mar+ in December 2023 would mean the contract expiring on March 2024.

For instance, consider the rolling schedule of E-mini S&P 500 futures:
RIC Jan Feb Mar Apr May Jun Jul Aug Sep Oct Nov Dec
0#ES: Mar Mar Mar Jun Jun Jun Sep Sep Sep Dec Dec Dec

For all CALCULATION DAYS in November the ACTIVE CONTRACT would be the one expiring in December the
same year, and the NEXT ACTIVE CONTRACT would expire in March the following year.

RIC Jan Feb Mar Apr May Jun Jul Aug Sep Oct Nov Dec
0#ES: Mar Jun Jun Jun Sep Sep Sep Dec Dec Dec Mar+ Mar+
21
Version 1.0 – 02 November 2023
1.3. Weighting of the Contracts
In relation to CALCULATION DAY 𝑡 the ROLL ANCHOR “𝑎𝑛𝑐ℎ𝑜𝑟𝑡” is defined as follows:

Where the column titled “Roll Anchor” in Table 2 (Futures Components Parameters) of Appendix 2
is set to “First Notice”, the ROLL ANCHOR 𝑎𝑛𝑐ℎ𝑜𝑟𝑡 is set to the [FIRST NOTICE DAY] of the ACTIVE
CONTRACT.
Where the column titled “Roll Anchor” in Table 2 (Futures Components Parameters) of Appendix 2
is set to “Expiry”, the ROLL ANCHOR 𝑎𝑛𝑐ℎ𝑜𝑟𝑡 is set to the [EXPIRATION DAY] of the ACTIVE CONTRACT.
Once ROLL ANCHOR 𝑎𝑛𝑐ℎ𝑜𝑟𝑡 as of CALCULATION DAY 𝑡 is determined, the ROLL START “𝑅𝑜𝑙𝑙𝑆𝑡𝑎𝑟𝑡𝑡” in
relation to CALCULATION DAY 𝑡 is set to where (i) the “Roll Offset” identified in Table 2 (Futures Components
Parameters) of Appendix 2 is a negative number, a number of Calculation Days immediately preceding
ROLL ANCHOR 𝑎𝑛𝑐ℎ𝑜𝑟𝑡 equal to the sum of (A) the absolute value of such “Roll Offset” and (ii) one, and (B)
“Roll Offset” is a positive number, a number of Calculation Days immediately following the ROLL ANCHOR
𝑎𝑛𝑐ℎ𝑜𝑟𝑡 equal to such “Roll Offset” minus one.

Lastly, ROLL END “𝑅𝑜𝑙𝑙𝐸𝑛𝑑𝑡” as of CALCULATION DAY 𝑡 is set to be the number of CALCULATION DAYS
immediately following the ROLL START 𝑅𝑜𝑙𝑙𝑆𝑡𝑎𝑟𝑡𝑡 equal to the numeric value identified in the column
“Roll Days” in Table 2 (Futures Components Parameters) of Appendix 2.
In relation to CALCULATION DAY 𝑡 the “ACTIVE CONTRACT WEIGHT” is calculated as follows:

In relation to CALCULATION DAY 𝑡 the “ACTIVE CONTRACT WEIGHT” is calculated as follows:

𝑤𝑡𝐴𝑐𝑡𝑖𝑣𝑒=
{
0 if 𝑡≤ 𝑅𝑜𝑙𝑙𝑆𝑡𝑎𝑟𝑡𝑡
#𝐶𝐷𝑎𝑦𝑠𝑡,𝑅𝑜𝑙𝑙𝐸𝑛𝑑𝑡
𝑅𝑜𝑙𝑙𝐷𝑎𝑦𝑠
if 𝑅𝑜𝑙𝑙𝑆𝑡𝑎𝑟𝑡𝑡<𝑡<𝑅𝑜𝑙𝑙𝐸𝑛𝑑𝑡
0 if 𝑅𝑜𝑙𝑙𝐸𝑛𝑑𝑡≤𝑡
Where:
𝑅𝑜𝑙𝑙𝐷𝑎𝑦𝑠:has the meaning given to it in the column titled “Roll Days” in Table 2 (Futures Components
Parameters) of Appendix 2.
#𝐶𝐷𝑎𝑦𝑠𝑡,𝑅𝑜𝑙𝑙𝐸𝑛𝑑𝑡: is the number of CALCULATION DAYS between CALCULATION DAY 𝑡 (including) and
𝑅𝑜𝑙𝑙𝐸𝑛𝑑𝑡 (excluding).
In relation to CALCULATION DAY 𝑡 the “NEXT ACTIVE CONTRACT WEIGHT” is calculated as follows:
𝑤𝑡𝑁𝑒𝑥𝑡= 1 −𝑤𝑡𝐴𝑐𝑡𝑖𝑣𝑒
To illustrate the above, consider the rolling schedule of FUTURES COMPONENTS comprising of E-mini S&P
500 futures:
RIC Roll Anchor Roll Offset Roll Days
0#ES: Expiry - 6 5
In the S&P 500 futures case, ROLL ANCHOR would be set to the ACTIVE CONTRACT’S EXPIRATION DAY. Therefore,
ROLL START would be the seventh day before expiration, illustrated in the below table.

22
Version 1.0 – 02 November 2023
Weekday Mon Tue Wed Thu Fri Mon Tue Wed Thu Fri
Named Day
Roll
Start
Roll
End
Roll
Anchor
ACTIVE CONTRACT
WEIGHT
100% 100% 100% 80% 60% 40% 20% 0% 0% 0%
NEXT ACTIVE
CONTRACT WEIGHT
0% 0% 0% 20% 40% 60% 80% 100% 100% 100%
2. Calculation of the Rolling Future Level
2.1. Formula
On FUTURES COMPONENTS START DATE 𝑡 0 , ROLLING FUTURES LEVEL for each FUTURES COMPONENTS is defined as:
𝑅𝐹𝐿𝑒𝑣𝑒𝑙𝑡 0 =𝑆𝑡𝑎𝑟𝑡𝐿𝑒𝑣𝑒𝑙

Where:
𝑆𝑡𝑎𝑟𝑡𝐿𝑒𝑣𝑒𝑙:has the meaning given to it in the column titled “Start Level” in Table 3 (Calculation
Parameters) of Appendix 2.

On any CALCULATION DAY 𝑡 after FUTURES COMPONENTS START DATE, the ROLLING FUTURES LEVEL is calculated as
follows:
𝑅𝐹𝐿𝑒𝑣𝑒𝑙𝑡=𝑅𝐹𝐿𝑒𝑣𝑒𝑙𝑡− 1 ×( 1 +𝐹𝑢𝑡𝑢𝑟𝑒𝑠𝑅𝑒𝑡𝑢𝑟𝑛𝑡)
Where:
𝑅𝐹𝐿𝑒𝑣𝑒𝑙𝑡− 1 : is the ROLLING FUTURES LEVEL on the CALCULATION DAY immediately preceding CALCULATION DAY
𝑡.
𝐹𝑢𝑡𝑢𝑟𝑒𝑅𝑒𝑡𝑢𝑟𝑛𝑡: is the FUTURES RETURN as of CALCULATION DAY 𝑡 as defined in section 2.1.1.

F U T U R ES R E T U R N
The FUTURES RETURN as of CALCULATION DAY 𝑡 is calculated as follows:

𝐹𝑢𝑡𝑢𝑟𝑒𝑠𝑅𝑒𝑡𝑢𝑟𝑛𝑡=(𝑤𝑡𝐴𝑐𝑡𝑖𝑣𝑒×(
𝑃𝑥𝑡𝐴𝑐𝑡𝑖𝑣𝑒
𝑃𝑥𝑡𝐴𝑐𝑡𝑖𝑣𝑒− 1
− 1 )+𝑤𝑡𝑁𝑒𝑥𝑡×(
𝑃𝑥𝑡𝑁𝑒𝑥𝑡
𝑃𝑥𝑡𝑁𝑒𝑥𝑡− 1
− 1 ))×𝐹𝑋𝐶𝑜𝑛𝑣𝑒𝑟𝑠𝑖𝑜𝑛𝑡
Where:
𝑤𝑡𝐴𝑐𝑡𝑖𝑣𝑒: is the ACTIVE CONTRACT WEIGHT as of CALCULATION DAY 𝑡 as defined in section 1.3.
𝑤𝑡𝑁𝑒𝑥𝑡: is the NEXT ACTIVE CONTRACT WEIGHT as of CALCULATION DAY 𝑡 as defined in section 1.3.
𝑃𝑥𝑡𝐴𝑐𝑡𝑖𝑣𝑒: is the FUTURES LEVEL of the ACTIVE CONTRACT as of CALCULATION DAY 𝑡 as defined in section 2 .1. 2.
𝑃𝑥𝑡𝑁𝑒𝑥𝑡: is the FUTURES LEVEL of the NEXT ACTIVE CONTRACT as of CALCULATION DAY 𝑡 as defined in section
2 .1. 2.
𝑃𝑥𝑡𝐴𝑐𝑡𝑖𝑣𝑒− 1 : is the FUTURES LEVEL of the ACTIVE CONTRACT as of CALCULATION DAY immediately preceding
CALCULATION DAY 𝑡 as defined in section 2 .1. 2.

23
Version 1.0 – 02 November 2023
𝑃𝑥𝑡𝑁𝑒𝑥𝑡− 1 : is the FUTURES LEVEL of the NEXT ACTIVE CONTRACT as of CALCULATION DAY immediately preceding
CALCULATION DAY 𝑡 as defined in section 2 .1..
𝐹𝑋𝐶𝑜𝑛𝑣𝑒𝑟𝑠𝑖𝑜𝑛𝑡: is the FX CONVERSION as of CALCULATION DAY 𝑡 as defined in Section 2 .1. 3.

F U T U R ES L E V E L
If the column titled “Price Definition” in Table 2 (Futures Component Parameters) of Appendix 2 is set to
“Settlement”, the FUTURES LEVEL of the FUTURES CONTRACT c as of CALCULATION DAY 𝑡 is set to its SETTLEMENT
LEVEL provided by the EXCHANGE:
𝑃𝑥𝑡𝑐=𝑆𝑒𝑡𝑡𝑙𝑒𝑚𝑒𝑛𝑡𝑡𝑐

F X C O N V E R S I O N
In case INDEX CURRENCY equals FUTURES CURRENCY, the FX CONVERSION as of CALCULATION DAY 𝑡 is set:
𝐹𝑋𝐶𝑜𝑛𝑣𝑒𝑟𝑠𝑖𝑜𝑛𝑡= 1
In case INDEX CURRENCY does not equal FUTURES CURRENCY, the FX CONVERSION as of CALCULATION DAY 𝑡 is
calculated as follows:

𝐹𝑋𝐶𝑜𝑛𝑣𝑒𝑟𝑠𝑖𝑜𝑛𝑡=
𝐹𝑋𝑡
𝐹𝑋𝑡− 1
Where:
𝐹𝑋𝑡: is the exchange rate to convert the FUTURES CURRENCY to the INDEX CURRENCY on CALCULATION DAY 𝑡 as
described in Section 1.5 (Prices and Calculation Frequency) of the Guideline.
𝐹𝑋𝑡− 1 : is the exchange rate to convert the FUTURES CURRENCY to the INDEX CURRENCY on CALCULATION DAY
immediately preceding CALCULATION DAY 𝑡 as described in Section 1.5 (Prices and Calculation Frequency)
of the Guideline.

Appendix 2 – Tables
Table 1 (Identifiers) is set out in section 2.2 (Index Components) of the Guideline

Table 2 (Futures Component Parameters)

Futures
Chain RIC
Exchange
MIC
Futures
Currency
Price
Definition
Interest
Rate Ric
IR
Compound
Method
IR Offset IR Day Count Roll Anchor Roll Offset Roll Days Portfolio
0#ES: XCME USD Settlement Expiry - 6 5 FALSE
0#NQ: XCME USD Settlement Expiry - 6 5 FALSE
0#TY: XCBT USD Settlement First Notice - 6 5 FALSE
0#TU: XCBT USD Settlement First Notice - 6 5 FALSE
0#URO: FCME-CMG USD Settlement Expiry - 6 5 FALSE
0#JY: FCME-CMG USD Settlement Expiry - 6 5 FALSE
0#NIY: XCME JPY Settlement Expiry - 6 5 FALSE
0#STXE: XEUR EUR Settlement Expiry - 6 5 FALSE
0#FGBL: XEUR EUR Settlement Expiry - 6 5 FALSE
25
Version 1.0 – 02 November 2023
Table 3 (Calculation Parameters)

Futures
Chain RIC
Futures Component Start
Date Start Level^
0#ES: (^2000) - 01 - 03 100
0#NQ: (^2002) - 04 - 01 100
0#TY: (^2002) - 06 - 03 100
0#TU: (^2002) - 06 - 03 100
0#URO: (^1999) - 01 - 04 100
0#JY: (^2002) - 07 - 01 100
0#NIY: (^2004) - 04 - 01 100
0#STXE: (^1999) - 01 - 04 100
0#FGBL: (^2002) - 10 - 01 100

26
Version 1.0 – 02 November 2023
Table 4 (Expiration Month of the Active Contract)

RIC Jan Feb Mar Apr May Jun Jul Aug Sep Oct Nov Dec
0#ES: Mar Mar Mar Jun Jun Jun Sep Sep Sep Dec Dec Dec
0#NQ: Mar Mar Mar Jun Jun Jun Sep Sep Sep Dec Dec Dec
0#TY: Mar Mar Jun Jun Jun Sep Sep Sep Dec Dec Dec Mar+
0#TU: Mar Mar Jun Jun Jun Sep Sep Sep Dec Dec Dec Mar+
0#URO: Mar Mar Mar Jun Jun Jun Sep Sep Sep Dec Dec Dec
0#JY: Mar Mar Mar Jun Jun Jun Sep Sep Sep Dec Dec Dec
0#NIY: Mar Mar Mar Jun Jun Jun Sep Sep Sep Dec Dec Dec
0#STXE: Mar Mar Mar Jun Jun Jun Sep Sep Sep Dec Dec Dec
0#FGBL: Mar Mar Mar Jun Jun Jun Sep Sep Sep Dec Dec Dec
27
Version 1.0 – 02 November 2023
Table 5 (Expiration Month of the Next Active Contract)

RIC Jan Feb Mar Apr May Jun Jul Aug Sep Oct Nov Dec
0#ES: Mar Jun Jun Jun Sep Sep Sep Dec Dec Dec Mar+ Mar+
0#NQ: Mar Jun Jun Jun Sep Sep Sep Dec Dec Dec Mar+ Mar+
0#TY: Mar Jun Jun Jun Sep Sep Sep Dec Dec Dec Mar+ Mar+
0#TU: Mar Jun Jun Jun Sep Sep Sep Dec Dec Dec Mar+ Mar+
0#URO: Mar Jun Jun Jun Sep Sep Sep Dec Dec Dec Mar+ Mar+
0#JY: Mar Jun Jun Jun Sep Sep Sep Dec Dec Dec Mar+ Mar+
0#NIY: Mar Jun Jun Jun Sep Sep Sep Dec Dec Dec Mar+ Mar+
0#STXE: Mar Jun Jun Jun Sep Sep Sep Dec Dec Dec Mar+ Mar+
0#FGBL: Mar Jun Jun Jun Sep Sep Sep Dec Dec Dec Mar+ Mar+
APPENDIX 3 - DEFINIT IONS
The definitions in this Appendix 3 (Definitions) shall apply for the purposes of Appendices 1 and 2 only and,
in the event of any inconsistency with terms defined in Section 5 (Definitions) of the Guideline, the
definitions in this Appendix 3 (Definitions) shall prevail.

“ACTIVE CONTRACT” means for a future in Table 2 and a CALCULATION DAY t , the FUTURES CONTRACT in which
the FUTURES COMPONENTS is invested on the first day of the calendar month of t , as per the roll schedule
given in Appendix 2, Table 4.
"ACTIVE CONTRACT WEIGHT" in relation to the ACTIVE CONTRACT and a CALCULATION DAY shall have the
meaning as defined in Appendix 1, Section 1.3.
“CALCULATION DAY” is every weekday from Monday to Friday. A day on which the EXCHANGE is not open for
general business is not a CALCULATION DAY
“EXCHANGE” shall have the meaning as defined in the column titled “Exchange MIC” in Table 2 (Futures
Component Parameters) of Appendix 2.
“EXPIRATION DAY” refers to the expiry date as announced by the EXCHANGE.
“FIRST NOTICE DAY” refers to the first notice date as announced by the EXCHANGE. It is determined as the
last day the ACTIVE CONTRACT of the future can be traded on the EXCHANGE.
“FUTURES CHAIN” is the set of FUTURES CONTRACTS that are related to a specific EXCHANGE and specific
underlying asset. A FUTURES CHAIN is identified in the column titled “Futures Chain RIC” in Table 2 (Futures
Component Parameters) of Appendix 2.
“FUTURES COMPONENTS” shall have the meaning as defined in Appendix 1, Section 1.1.
“FUTURES COMPONENTS START DATE” shall have the meaning given to it in the column titled “Futures
Component Start Date” in Table 3 (Calculation Parameters) of Appendix 2.
“FUTURES CONTRACTS” means a contract that confers an obligation to trade the underlying asset at a pre-
defined price on a pre-defined date in the future.
“FUTURES CURRENCY” shall have the meaning as defined in the column titled “Futures Currency” in Table
2 (Futures Component Parameters) of Appendix 2.
“FUTURES LEVEL” shall have the meaning as defined in Appendix 1, Section 2.1.4.
“FUTURES RETURN” shall have the meaning as defined in Appendix 1, Section 2.1.1.
“FX CONVERSION” shall have the meaning as defined in Appendix 1, Section 2.1.5.
“INTEREST RATE INSTRUMENT” is the instrument that is identified by its RIC as defined in the column titled
“INTEREST RATE RIC” in Table 2 (Futures Component Parameters) of Appendix 2.
“MIC” means Market Identifier Code.
“NEXT ACTIVE CONTRACT” shall have the meaning as defined in Appendix 1, Section 1.2 and as per Table 4
(Expiration Month of the Active Contract) of Appendix 2.

“NEXT ACTIVE CONTRACT WEIGHT” in relation to the NEXT ACTIVE CONTRACT and a CALCULATION DAY shall have
the meaning as defined in Appendix 1, Section 1.3.
“RIC” means Refinitiv Instrument Code.
“ROLL ANCHOR” shall have the meaning as defined in Appendix 1, Section 1.3.
“ROLL END” shall have the meaning as defined in Appendix 1, Section 1.3.
“ROLL START” shall have the meaning as defined in Appendix 1, Section 1.3.

29
Version 1.0 – 02 November 2023
The “SETTLEMENT LEVEL” in respect of a FUTURES CONTRACTS and a CALCULATION DAY is a security's final
regular-hours price at which the FUTURES CONTRACTS will reference at the end of each CALCULATION DAY and
upon its expiration published by the EXCHANGE and determined in accordance with the EXCHANGE
regulations.
“START LEVEL” shall have the meaning given to it in the column titled “Start Level” in Table 3 (Calculation
Parameters) of Appendix 2.
The “TRADING PRICE” in respect of a FUTURES CONTRACTS and a CALCULATION DAY is the most recent
published price at which the FUTURES CONTRACTS was traded on the respective EXCHANGE.
The “TRADED VOLUME” in respect of a FUTURES CONTRACTS and a CALCULATION DAY is the number of
contracts traded corresponding to the respective.

30
Version 1.0 – 02 November 2023
CONTACT

Solactive AG
German Index Engineering
Guiollettstr. 54
60325 Frankfurt am Main
Germany
Tel.: +49 (0) 69 719 160 00
Fax: +49 (0) 69 719 160 25
Email: info@solactive.com
Website: http://www.solactive.com

© Solactive AG

This is a offline tool, your data stays locally and is not send to any server!
Feedback & Bug Reports
