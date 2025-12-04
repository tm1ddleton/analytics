Version 1. 1
30 September 2025
HSBC US Variance Replication 1 Index

HSBC US Variance Replication 2 Index

TABLE OF CONTENTS
        -
     - Version 1.1 – 30 September
Introduction
Index Specifications
1.1. Scope of the Index
1.2. Identifiers and Publication
1.3. Initial Level of the Index
1.4. Prices and calculation frequency
1.5. Licensing
Index Selection
2.1. Selection of Index Components
2.1.1. Calculation of Variance Strike
2.1.2. Calculation of Variance Replication
2.2. Selection of the ElIgible Listed Options
2.2.1. Filtering of Eligible Listed Options
2.2.2. Listed Options Bid/Ask Prices
2.3. Selection of the Hedge Instrument
Rebalance 2.4. TWAP Calculation Methodology Error! Bookmark not defined.
3.1. Ordinary Rebalance
3.2. Extraordinary Rebalance.....................................................................................................................................
Calculation of the Index
4.1. Index Formula
4.1.1. Portfolio Mark-To-Market.........................................................................................................................
4.1.2. Continuing Option Portfolio
4.1.3. Cash Amount
4.1.4. Premium Paid
4.1.5. Exercise Values..........................................................................................................................................
4.1.6. Unwind Values
4.1.7. Delta Hedge Values
4.1.8. Delta Hedge Fee
4.2. Option Pricing Methodology
4.2.1. Payoff
4.2.2. Premium
4.2.3. Eligible Listed Option Implied Volatility
4.2.4. Option Greeks Calculation
4.2.5. Day Count Fraction
-
Version 1.1 – 30 September
4.2.6. Discount Factor
4.2.7. Maturity Selection
4.2.8. Forward
4.2.9. Discount Factor and Forward for an Eligible Listed Expiration Date
4.2.10. Implied Volatility
4.3. Accuracy..............................................................................................................................................................
4.4. Recalculation
4.5. Market Disruption
Miscellaneous..........................................................................................................................................................
5.1. Discretion
5.2. Methodology Review
5.3. Changes in Calculation Method
5.4. Termination
5.5. Index Committee
Definitions
Versioning
Contact..............................................................................................................................................................................
4
Version 1.1 – 30 September 2025
Introduction
This document (the “ GUIDELINE ”) is to be used as a guideline with regard to the composition,
calculation and maintenance of the following two indices: (i) HSBC US Variance Replication 1 Index,
and (ii) HSBC US Variance Replication 2 Index (each such index, the “ INDEX ”). References herein to
the “INDEX” shall refer to each INDEX individually and this GUIDELINE shall be construed accordingly.
Any amendments to the rules made to the GUIDELINE are approved by the INDEX COMMITTEE specified
in Section 5.5. The INDEX is calculated, administered and published by Solactive AG (“ SOLACTIVE ”)
assuming the role as administrator (the “ INDEX ADMINISTRATOR”) under the Regulation (EU)
2016/1011 (the “ BENCHMARK REGULATION ” or “ BMR ”). The name “Solactive” is trademarked.

HSBC Bank plc (the “INDEX OWNER”) owns the copyright and all other intellectual property rights in
the INDEX. Any use of these intellectual property rights may only be made with the prior written
consent of the INDEX OWNER.

The INDEX OWNER is not responsible for the actions or inactions of the INDEX ADMINISTRATOR in
accordance with the agreement on index administration and index calculation between the INDEX
OWNER and the INDEX ADMINISTRATOR and any other separate agreements between the INDEX OWNER
and the INDEX ADMINISTRATOR that have been or may be entered into from time to time.

The INDEX will be governed by the INDEX ADMINISTRATOR. The INDEX ADMINISTRATOR controls the
creation and operation of the INDEX, including (but not limited to) all stages and processes involved
in the production, calculation, maintenance, administration and dissemination of the INDEX.
Notwithstanding that the INDEX relies on information from third party sources, the INDEX
ADMINISTRATOR has primary responsibility for all aspects of the INDEX administration and
determination process.

In no event shall the INDEX OWNER be liable (whether directly or indirectly, in contract, tort or
otherwise) for any loss incurred by any person that arises out of or in connection with the INDEX,
including in relation to the performance by the INDEX OWNER of any part of its role in respect of
each INDEX, save in the case of gross negligence, fraud or wilful default. In no event, shall the INDEX
OWNER have any liability to any persons for any direct, indirect, special, punitive or consequential
damages (including lost profits) even if notified of the possibility of such damages.

With respect to any products linked to any index, the INDEX OWNER expressly disclaims all liability
for regulatory, juridical or reputational consequences suffered by any party in any transaction
connected with any INDEX.

The GUIDELINE and the policies and methodology documents referenced herein contain the
underlying principles and rules regarding the structure and operation of the INDEX. The INDEX
ADMINISTRATOR does not offer any explicit or tacit guarantee or assurance, neither pertaining to
the results from the use of the INDEX nor the level of the INDEX at any certain point in time nor in
any other respect. The INDEX ADMINISTRATOR strives to the best of its ability to ensure the
correctness of the calculation. There is no obligation for The INDEX ADMINISTRATOR – irrespective of
possible obligations to issuers – to advise third parties, including investors and/or financial
intermediaries, of any errors in the INDEX. The publication of the INDEX by The INDEX ADMINISTRATOR
does not constitute a recommendation for capital investment and does not contain any assurance
or opinion of The INDEX ADMINISTRATOR regarding a possible investment in a financial instrument
based on this INDEX.

5
Version 1.1 – 30 September 2025
The text uses defined terms which are formatted with “SMALL CAPS”. Such Terms shall have the
meaning assigned to them as specified in Section 6 (Definitions).

6
Version 1.1 – 30 September 2025
1. Index Specifications
1.1. Scope of the Index
Category Description
Asset Class Equity
Strategy
The INDEX is a rules-based strategy that aims to capture certain
elements of performance associated with over-the-counter
variance swaps. The INDEX notionally enters into a short
position on a portfolio comprised of five static sets of S&P 500
Index listed options, with each set having different expiration
dates (each set, a “ Variance Replication ”). Such options are
delta hedged on an hourly basis. The Vega notional traded is
equally divided amongst the five Variance Replications^1.
The INDEX is calculated on a notional basis. The investment
exposure provided by the INDEX to the options referenced in the
INDEX is purely synthetic and an investor in the INDEX will have
no rights in respect of any such options. For the avoidance of
doubt, any reference herein to options being “entered into” is
purely on a notional basis.
Regional Allocation North America
Table 1 Index Overview

1.2. Identifiers and Publication
The INDEX is published under the following identifiers:

Name ISIN
Index
Currency Type^ BBG ticker^ RIC^
HSBC US Variance Replication
1 Index
DE000SL0NRR5 USD Excess
Return
HSIESGU
Index
.HSIESGU
HSBC US Variance Replication
2 Index DE000SL0NRS3^ USD^
Excess
Return
HSIESGU
Index .HSIESGU^
HSBC US Variance Replication
1 Indicative Index DE000SL0NRR5^ USD^
Excess
Return
HSIOSGU
Index .HSIOSGU^
HSBC US Variance Replication
2 Indicative Index DE000SL0NRS3^ USD^
Excess
Return
HSIOSGU
Index
.HSIOSGU
Each INDEX is published on the website of the INDEX ADMINISTRATOR (www.solactive.com) and is, in
addition, available via the price marketing services of Boerse Stuttgart GmbH and may be distributed
to all of its affiliated vendors. Each vendor decides on an individual basis as to whether it will
distribute or display the INDEX via its information systems.

(^1) Prior to the TRANSITION DATE, there were three Variance Replications.

7
Version 1.1 – 30 September 2025
Any publication in relation to the INDEX (e.g. notices, amendments to the GUIDELINE) will be available
at the website of the INDEX ADMINISTRATOR: https://www.solactive.com/news/announcements/.

1.3. Initial Level of the Index
The initial level of the INDEX on the START DATE is 100. Historical values from the LIVE DATE will be
recorded in accordance with Article 8 of the BMR. Levels of the INDEX published for a period prior to
the LIVE DATE have been back-tested using exchange prices. Levels of the INDEX published for the
period falling prior to 1st August 2022 have been provided by the INDEX OWNER to the INDEX
ADMINISTRATOR. The INDEX OWNER has obtained the listed options available from REFINITIV and
calculated the levels of the INDEX for the period of START DATE to LIVE DATE.

1.4. Prices and calculation frequency
The levels of the HSBC US VARIANCE REPLICATION 1 INDEX and HSBC US VARIANCE REPLICATION 2 INDEX are
calculated in respect of each CALCULATION DAY t and is published at 09:00 a.m. CET on the CALCULATION
DAY immediately following CALCULATION DAY t. The levels of the HSBC US VARIANCE REPLICATION 1
INDICATIVE INDEX and HSBC US VARIANCE REPLICATION 2 INDICATIVE INDEX are calculated in respect of each
CALCULATION DAY t and is published at 05:00 p.m. EST on the CALCULATION DAY t.

1.5. Licensing
Licenses to use the INDEX as the underlying value for financial instruments, investment funds and
financial contracts may be issued to stock exchanges, banks, financial services providers and
investment houses by the INDEX OWNER.

8
Version 1.1 – 30 September 2025
2. Index Selection
2.1. Selection of Index Components
On each CALCULATION DAY t that is an EXPIRATION DATE in respect of a LISTED OPTION but not a HALF TRADING
DAY, a synthetic portfolio of several Variance Replications (“𝑉𝑅𝑡𝑖”) is notionally traded in connection
with the INDEX.

The parameters of the INDEX, and the characteristics of the traded OPTIONS in respect of each of the
Variance Replications, differ depending upon the CALCULATION DAY t in respect of which notional
trading occurs:

(a) Effective up to, but excluding, the TRANSITION DATE, the parameters of the INDEX and the
characteristics of traded OPTIONS in respect of each of the Variance Replications, are defined
below in: “ Table 2: Old Index Parameters ” and “ Table 4 : Old Options’ Characteristics ”;
(b) Effective from, and including, the TRANSITION DATE, the parameters of the INDEX and the
characteristics of traded OPTIONS in respect of each of the Variance Replications, are defined
below in: “ Table 3: New Index Parameters ” and “ Table 5: New Options’ Characteristics ”.
PARAMETER GUIDELINES
NOTATION
HSIESGU1 HSIESGU
UNDERLYING INDEX SPX Index SPX Index
LISTED OPTIONS S&P 500 Weekly Options S&P 500 Weekly Options
VEGA SIZING 𝑉𝑆𝑉𝑅 (^0) .01%×^5
3 0 .02%×
5
3
STRIKE STEP 𝑆𝑡𝑒𝑝𝑉𝑅 (^25 )
DELTA HEDGE FEE 𝑑ℎ𝑓 0.01% 0.01%
FRICTION 𝑓 0.15% 0.15%
Table 2 : Old Index Parameters
PARAMETER GUIDELINES
NOTATION
HSIESGU1 HSIESGU
UNDERLYING INDEX SPX Index SPX Index
LISTED OPTIONS S&P 500 Weekly Options S&P 500 Weekly Options
VEGA SIZING 𝑉𝑆𝑉𝑅 0.01% 0.02%
STRIKE STEP 𝑆𝑡𝑒𝑝𝑉𝑅 25 25
DELTA HEDGE FEE 𝑑ℎ𝑓 0.01% 0.01%

9
Version 1.1 – 30 September 2025
FRICTION 𝑓 0.15% 0.15%
Table 3 : New Index Parameters

VARIANCE REPLICATION NAME GUIDELINES
NOTATION
𝑉𝑅 (^1) 𝑡 𝑉𝑅 (^2) 𝑡 𝑉𝑅 3
TRADE DATE 𝑇𝐷𝑉𝑅 CALCULATION DAY t
EXPIRATION DATE 𝑇𝐸𝑉𝑅^ M1t^ M2t^ M3t^
UNWIND DATE 𝑇𝑈𝑉𝑅^ M1t^ M2t^ M3t^
WEIGHT 𝑤𝑉𝑅^ −40%^ −40%^ −20%^
LOWER EXECUTION BOUND 𝐿𝐸𝐵𝑉𝑅^ 𝐿𝐸𝐵^1 𝑡− 1^ 𝐿𝐸��^2 𝑡− 1^ 𝐿𝐸𝐵^3 𝑡− 1^
UPPER EXECUTION BOUND ��𝐸𝐵𝑉𝑅 𝑈𝐸𝐵 (^1) 𝑡− 1 𝑈𝐸𝐵 (^2) 𝑡− 1 𝑈𝐸𝐵 (^3) 𝑡− 1
LOWER THEORETICAL BOUND 𝐿𝑇𝐵𝑉𝑅 ��𝑇𝐵 (^1) 𝑡− 1 𝐿𝑇𝐵 (^2) 𝑡− 1 𝐿𝑇𝐵 (^3) 𝑡− 1
UPPER THEORETICAL BOUND 𝑈𝑇𝐵𝑉𝑅 𝑈𝑇𝐵 (^1) 𝑡− 1 𝑈𝑇𝐵 (^2) 𝑡− 1 𝑈𝑇𝐵 (^3) 𝑡− 1
VARIANCE STRIKE 𝐾𝑣𝑎𝑟𝑉𝑅^ 𝐾𝑣𝑎𝑟^1 𝑡− 1 𝐾𝑣𝑎𝑟^2 𝑡− 1 𝐾𝑣𝑎𝑟^3 𝑡− 1
Table 4 : Old Options’ Characteristics
VARIANCE REPLICATION
NAME
GUIDELINES
NOTATION
𝑉𝑅 (^1) 𝑡 𝑉𝑅 (^2) 𝑡 𝑉𝑅 3 𝑉𝑅 (^4) 𝑡 𝑉𝑅 (^5) 𝑡
TRADE DATE 𝑇𝐷𝑉𝑅 CALCULATION DAY t
EXPIRATION DATE 𝑇𝐸𝑉𝑅 M1t M2t M3t M4t M5t
UNWIND DATE 𝑇𝑈𝑉𝑅 M1t M2t M3t M4t M5t
WEIGHT 𝑤𝑉𝑅 −20%
LOWER EXECUTION BOUND 𝐿𝐸𝐵𝑉𝑅 𝐿𝐸𝐵 (^1) 𝑡− 1 𝐿𝐸𝐵 (^2) 𝑡− 1 𝐿𝐸𝐵 (^3) 𝑡− 1 𝐿��𝐵 (^4) 𝑡− 1 𝐿𝐸𝐵 (^5) 𝑡− 1
UPPER EXECUTION BOUND 𝑈𝐸𝐵𝑉𝑅 𝑈𝐸𝐵 (^1) 𝑡− 1 𝑈𝐸𝐵 (^2) 𝑡− 1 𝑈𝐸𝐵 (^3) 𝑡− 1 𝑈𝐸𝐵 (^4) 𝑡− 1 𝑈𝐸𝐵 (^5) 𝑡− 1
LOWER THEORETICAL
BOUND
𝐿𝑇𝐵𝑉𝑅 𝐿𝑇𝐵 (^1) 𝑡− 1 𝐿𝑇𝐵 (^2) 𝑡− 1 𝐿𝑇𝐵 (^3) 𝑡− 1 𝐿𝑇�� (^4) 𝑡− 1 𝐿𝑇𝐵 (^5) 𝑡− 1
UPPER THEORETICAL
BOUND
𝑈𝑇𝐵𝑉𝑅 𝑈𝑇𝐵 (^1) 𝑡− 1 𝑈𝑇𝐵 (^2) 𝑡− 1 𝑈𝑇𝐵 (^3) 𝑡− 1 𝑈𝑇𝐵 (^4) 𝑡− 1 𝑈𝑇𝐵 (^5) 𝑡− 1
VARIANCE STRIKE 𝐾𝑣𝑎𝑟𝑉𝑅 𝐾𝑣𝑎𝑟 (^1) ��− 1 𝐾𝑣𝑎𝑟 (^2) 𝑡− 1 𝐾𝑣𝑎𝑟 (^3) 𝑡− 1 𝐾𝑣��𝑟 (^4) 𝑡− 1 𝐾𝑣𝑎𝑟 (^5) 𝑡− 1
Table 5 : New Options’ Characteristics

10
Version 1.1 – 30 September 2025
With:

𝑀𝑖𝑡: for each i from 1 to 5, means the i-th ELIGIBLE LISTED EXPIRATION DATE falling after CALCULATION DAY
t, provided that for the purposes of day counting any date that is not a CALCULATION DAY shall be
excluded from the counting process.

𝐿𝐸𝐵𝑖𝑡− 1 : for each i from 1 to 5, is the Lower Execution Bound for Variance Replication 𝑉𝑅𝑖𝑡,
computed on CALCULATION DAY t- 1 , and equal to the STRIKE PRICE of a PUT OPTION with EXPIRATION DATE
𝑀𝑖𝑡, such that the DELTA as defined in Section 4.2.4 is equal to -2% (with the following boundaries
70%×𝑆𝑡− 1 and 𝑆𝑡− 1 ).

𝑈𝐸𝐵𝑖𝑡− 1 : for each i from 1 to 5, is the Upper Execution Bound for Variance Replication 𝑉𝑅𝑖𝑡,
computed on CALCULATION DAY t-1, and equal to the STRIKE PRICE of a CALL OPTION with EXPIRATION DATE
𝑀𝑖𝑡, such that the DELTA as defined in Section 4.2.4 is equal to +2% (with the following boundaries
𝑆𝑡− 1 and 130% ×𝑆𝑡− 1 ).

𝐿𝑇𝐵𝑖𝑡− 1 : for each i from 1 to 5, is the Lower Theoretical Bound for Variance Replication 𝑉𝑅𝑖𝑡,
computed on CALCULATION DAY t-1, calculated according to the following formula:

𝐿𝑇𝐵𝑖𝑡− 1 =80%×𝑆𝑡− 1
𝑈𝑇𝐵𝑖𝑡− 1 : for each i from 1 to 5, is the Upper Theoretical Bound for Variance Replication 𝑉𝑅𝑖𝑡,
computed on CALCULATION DAY t-1, calculated according to the following formula:

𝑈𝑇𝐵𝑖𝑡− 1 =110%×𝑆𝑡− 1
𝑆𝑡− 1 : the closing level of the UNDERLYING INDEX on CALCULATION DAY t- 1.

𝐾𝑣𝑎��𝑖𝑡− 1 : for each i from 1 to 5, is the VARIANCE STRIKE for Variance Replication 𝑉𝑅𝑖𝑡 computed on
CALCULATION DAY t-1.

2.1.1. Calculation of Variance Strike
On CALCULATION DAY t, for each i from 1 to 5 the VARIANCE STRIKE 𝐾𝑣𝑎𝑟𝑖𝑡− 1 for Variance Replication
𝑉𝑅𝑖𝑡 is computed according to the following formula:

𝐾𝑣𝑎𝑟𝑖𝑡− 1 =√
2 ×(𝑃𝑢𝑡𝐶𝑜𝑛𝑡𝑟𝑖𝑏𝑡− 1 ,𝑀𝑖𝑡 +𝐶𝑎𝑙𝑙𝐶𝑜𝑛𝑡𝑟𝑖𝑏𝑡− 1 ,𝑀𝑖𝑡 +𝑆𝑡𝑟𝑎𝑑𝑑𝑙𝑒𝐶𝑜𝑛𝑡𝑟��𝑏𝑡− 1 ,𝑀𝑖𝑡 )
𝐷𝐶𝐹(𝑡− 1 ,𝑀𝑖𝑡)
Where:

��𝑢𝑡𝐶𝑜𝑛𝑡𝑟𝑖𝑏𝑡− 1 ,𝑀𝑖𝑡 = ∑
𝐴𝑏𝑠(𝐾𝑗−��𝑗− 1 )
𝐾𝑗^2
𝐿𝑖𝑠𝑡𝑒𝑑𝑃𝑢𝑡𝑡𝑀𝑖𝑑− 1 ,𝑀��𝑡,𝐾𝑗
|𝐸𝐿𝑆𝑃𝑡− 1 |
𝑗= 1
𝐾𝑗<𝐾 0
𝐾𝑗≥ min (𝐿𝑇𝐵𝑖𝑡− 1 ,𝐿𝐸𝐵𝑖𝑡− 1 )
And

11
Version 1.1 – 30 September 2025
𝐶𝑎𝑙𝑙𝐶𝑜𝑛𝑡𝑟𝑖𝑏𝑡− 1 ,𝑀𝑖𝑡 = ∑
𝐴𝑏𝑠(𝐾𝑗−𝐾𝑗− 1 )
𝐾𝑗^2
𝐿𝑖𝑠��𝑒𝑑𝐶𝑎𝑙𝑙𝑡𝑀𝑖𝑑− 1 ,𝑀𝑖𝑡,𝐾𝑗
|𝐸𝐿𝑆𝐶𝑡− 1 |
𝑗= 1
𝐾𝑗>𝐾 0
𝐾𝑗≤max (𝑈𝑇𝐵𝑖𝑡− 1 ,𝑈𝐸𝐵𝑖𝑡− 1 )
And

𝑆𝑡𝑟𝑎𝑑𝑑𝑙𝑒𝐶𝑜𝑛𝑡𝑟𝑖𝑏𝑡− 1 ,𝑀𝑖𝑡 =
0. 5 ×𝑆𝑡𝑒𝑝𝑉𝑅
𝐾 02
(𝐿𝑖𝑠𝑡𝑒��𝐶𝑎𝑙𝑙𝑡𝑀𝑖𝑑− 1 ,𝑀𝑖𝑡,𝐾 0 +𝐿𝑖𝑠𝑡𝑒𝑑𝑃��𝑡𝑡𝑀𝑖𝑑− 1 ,𝑀𝑖𝑡,𝐾 0 )
With

𝐾 0 =𝐶𝐿𝐹(��𝑤𝑑𝑡− 1 ,𝑀𝑖𝑡,𝑆𝑡𝑒𝑝𝑉𝑅)
With:

𝐸𝐿𝑆𝑃𝑡− 1 : the Eligible Listed Strikes Put is a subset of the ELIGIBLE LISTED STRIKES on CALCULATION DAY t-
1., satisfying the following two conditions: (i) the corresponding OPTION is a PUT OPTION, and (ii) the
STRIKE PRICE is a multiple of 𝑆𝑡𝑒𝑝𝑉𝑅 away from 𝐾 0. The resulting subset is sorted in descending order.

𝐸𝐿𝑆𝐶𝑡− 1 : the Eligible Listed Strikes Call is a subset of the ELIGIBLE LISTED STRIKES on CALCULATION DAY t-
1., satisfying the following two conditions: (i) the corresponding OPTION is a CALL OPTION, and (ii) the
STRIKE PRICE is a multiple of 𝑆𝑡𝑒𝑝𝑉𝑅 away from 𝐾 0. The resulting subset is sorted in ascending order.

|𝐸𝐿𝑆𝑃𝑡− 1 | : is the cardinal of the set Eligible Listed Strikes Put 𝐸𝐿𝑆𝑃𝑡− 1 as defined above.

|𝐸𝐿𝑆𝐶𝑡− 1 | : is the cardinal of the set Eligible Listed Strikes Call 𝐸𝐿𝑆𝐶𝑡− 1 as defined above.

𝑀𝑖𝑡: for each i from 1 to 5, means the i-th ELIGIBLE LISTED EXPIRATION DATE falling strictly after
CALCULATION DAY t.

𝐿𝑖𝑠𝑡𝑒𝑑𝐶𝑎𝑙𝑙𝑡𝑀𝑖𝑑,𝑀,𝐾 : means the LISTED MID PRICE, on CALCULATION DAY t, for a CALL OPTION with EXPIRATION
DATE 𝑀 and STRIKE PRICE 𝐾, as defined in Section 2.2.2.

𝐿𝑖𝑠𝑡𝑒𝑑𝑃𝑢𝑡𝑡𝑀𝑖𝑑,𝑀,𝐾 : means the LISTED MID PRICE, on CALCULATION DAY t, for a PUT OPTION with EXPIRATION
DATE 𝑀 and STRIKE PRICE 𝐾, as defined in Section 2.2.2.

𝐹𝑤𝑑��,𝑀 : the FORWARD for EXPIRATION DATE 𝑀 computed on CALCULATION DAY t according to Section
4.2.9.

𝑆𝑡𝑒𝑝𝑉𝑅 : is the STRIKE STEP for Variance Replication VR as defined in “ Table 2: Old Index Parameters ”
and “ Table 3: New Index Parameters ”.

𝐷𝐶𝐹(𝑡,𝑀𝑖𝑡) : means DAY COUNT FRACTION, in respect of EXPIRATION DATE 𝑀𝑖𝑡 as of CALCULATION DAY t,
computed as (i) the number of CALCULATION DAYS from (and including) CALCULATION DAY t to (but
excluding) EXPIRATION DATE 𝑀𝑖𝑡 and (ii) divided by 252.

𝐶𝐿𝐹(𝐹𝑤𝑑𝑡,𝑀,𝑠𝑡𝑒𝑝) : means the Closest Listed Forward and is the listed STRIKE PRICE for the EXPIRATION

DATE 𝑀 available on CALCULATION DAY t, which is the closest to 𝑟𝑜𝑢𝑛𝑑(𝐹𝑤𝑑𝑠𝑡𝑒𝑝𝑡,𝑀)×𝑠𝑡𝑒𝑝, where

𝑟𝑜𝑢𝑛��(𝑥) is the closest integer to 𝑥

12
Version 1.1 – 30 September 2025
2.1.2. Calculation of Variance Replication
On CALCULATION DAY t, for each i (ranging from 1 to 5) the Variance Replication 𝑉𝑅𝑖𝑡 is comprised of
the following OPTIONS, grouped into three sets:

(i) Put Option Set,

(ii) Call Option Set, and

(iii) Straddle Option Set,

each set consisting of pairs of UNITS and OPTIONS:

Put Option Set:

𝑃𝑢𝑡 𝑂𝑝𝑡𝑖𝑜𝑛 𝑆𝑒𝑡={(𝑈𝑛𝑖𝑡𝑃𝑢𝑡𝑗,𝑀𝑖𝑡,𝐾𝑣𝑎𝑟𝑖𝑡− 1 ,𝑃𝑢𝑡𝑀𝑖𝑡,𝐾𝑗,𝐾𝑣𝑎𝑟𝑖𝑡− 1 ) | 𝑗∈⟦ 1 ,|𝐸𝐿𝑆𝑃𝑡− 1 |⟧ ∧𝐾𝑗<𝐾 0 ∧ 𝐾𝑗≥𝐿𝐸𝐵𝑖𝑡− 1 }
Where:

𝑈𝑛𝑖𝑡𝑃��𝑡𝑗,𝑀𝑖𝑡,𝐾𝑣𝑎𝑟𝑖𝑡− 1 =𝐼𝑛𝑑𝑒𝑥𝑡𝑇𝑅− 1 ×
100 ×𝐴𝑏𝑠(𝐾𝑗−𝐾𝑗− 1 ) ×𝑉𝑆𝑉𝑅×𝑤𝑉𝑅
𝐾𝑣𝑎𝑟𝑖𝑡− 1 × 𝐾𝑗^2 ×𝐷𝐶𝐹(𝑡− 1 ,𝑀𝑖𝑡)
+𝐴𝑑��𝑜𝑛𝑈𝑛𝑖𝑡𝑃𝑢𝑡×𝟙{𝐾𝑗=𝐿𝑃𝑆}
Call Option Set:

𝐶𝑎𝑙𝑙 𝑂𝑝𝑡𝑖𝑜𝑛 𝑆𝑒𝑡={(𝑈𝑛𝑖𝑡𝐶𝑎𝑙��𝑗,𝑀𝑖𝑡,𝐾𝑣𝑎𝑟𝑖𝑡− 1 ,𝐶𝑎𝑙𝑙𝑀𝑖𝑡,𝐾𝑗,𝐾𝑣𝑎𝑟𝑖𝑡− 1 ) | 𝑗∈⟦ 1 ,|𝐸𝐿𝑆𝐶𝑡− 1 |⟧ ∧𝐾𝑗>𝐾 0 ∧ 𝐾𝑗≤𝑈𝐸𝐵𝑖𝑡− 1 }
Where:

𝑈𝑛𝑖𝑡𝐶𝑎𝑙𝑙𝑗,𝑀𝑖𝑡,𝐾𝑣𝑎𝑟𝑖𝑡− 1
=𝐼𝑛𝑑𝑒𝑥𝑡𝑇𝑅− 1 ×
100 ×𝐴𝑏𝑠(𝐾𝑗−𝐾𝑗− 1 ) ×𝑉𝑆𝑉𝑅×𝑤𝑉𝑅
𝐾𝑣��𝑟𝑖𝑡− 1 × 𝐾𝑗^2 ×𝐷𝐶𝐹(𝑡− 1 ,𝑀𝑖𝑡)
+𝐴𝑑𝑑𝑜𝑛𝑈𝑛𝑖𝑡𝐶𝑎𝑙𝑙×𝟙{𝐾𝑗=𝐻𝐶𝑆}
Straddle Option Set:

𝑆𝑡𝑟𝑎𝑑𝑑𝑙𝑒 𝑂𝑝𝑡𝑖𝑜𝑛 𝑆𝑒𝑡
={
(𝑈𝑛𝑖𝑡𝑆𝑡𝑟𝑎𝑑𝑑𝑙𝑒𝑀𝑖𝑡,𝐾𝑣𝑎𝑟𝑖𝑡− 1 ,𝐶𝑎𝑙𝑙𝑀𝑖𝑡,𝐾 0 ,𝐾𝑣𝑎𝑟𝑖𝑡− 1 ),(𝑈𝑛𝑖𝑡𝑆𝑡𝑟𝑎𝑑𝑑𝑙𝑒𝑀𝑖𝑡,𝐾𝑣𝑎𝑟𝑖𝑡− 1 ,𝑃𝑢𝑡𝑀𝑖𝑡,�� 0 ,𝐾𝑣𝑎𝑟𝑖𝑡− 1 )}^
Where:

𝑈𝑛𝑖𝑡𝑆𝑡𝑟𝑎��𝑑𝑙𝑒𝑀𝑖𝑡,𝐾𝑣𝑎𝑟𝑖𝑡− 1 = 𝐼𝑛𝑑𝑒𝑥𝑡𝑇𝑅− 1 ×
50 × 𝑆𝑡𝑒𝑝𝑉𝑅 ×𝑉𝑆𝑉𝑅×𝑤𝑉𝑅
𝐾𝑣𝑎𝑟𝑖��− 1 × 𝐾 02 × 𝐷𝐶𝐹(𝑡− 1 ,𝑀𝑖𝑡)
And where:

��𝑑𝑑𝑜𝑛𝑄𝑡𝑦𝑃𝑢𝑡 : is the sum of the quantities of the theoretical PUT OPTIONS that are not executed in
the Variance Replication, being a value computed according to the following formula:

𝐴𝑑��𝑜𝑛𝑈𝑛𝑖𝑡𝑃𝑢𝑡= ∑ 𝐼𝑛𝑑𝑒𝑥𝑡𝑇𝑅− 1 ×
100 ×𝐴𝑏𝑠(𝐾𝑗−𝐾𝑗− 1 ) ×𝑉𝑆𝑉𝑅×𝑤𝑉𝑅
𝐾𝑣𝑎𝑟𝑖��− 1 × 𝐾𝑗^2 ×𝐷𝐶𝐹(𝑡− 1 ,𝑀𝑖𝑡)
|𝐸𝐿𝑆𝑃𝑡− 1 |
𝑗= 1
𝐾𝑗<𝐾 0
𝐾𝑗≥𝐿𝑇𝐵𝑖𝑡− 1
𝐾𝑗<𝐿𝐸𝐵𝑖𝑡− 1
𝐴𝑑𝑑𝑜𝑛𝑄𝑡𝑦𝐶𝑎𝑙𝑙 : is the sum of the quantities of the theoretical CALL OPTIONS that are not executed
in the Variance Replication, being a value computed according to the following formula:

13
Version 1.1 – 30 September 2025
𝐴𝑑𝑑𝑜𝑛𝑈𝑛𝑖𝑡𝐶𝑎𝑙𝑙= ∑ 𝐼𝑛𝑑𝑒𝑥𝑡𝑇𝑅− 1 ×
100 ×��𝑏𝑠(𝐾𝑗−𝐾𝑗− 1 ) ×𝑉𝑆𝑉𝑅×𝑤𝑉𝑅
𝐾𝑣𝑎𝑟𝑖𝑡− 1 × 𝐾𝑗^2 ×𝐷𝐶𝐹(𝑡− 1 ,𝑀𝑖𝑡)
|𝐸𝐿𝑆𝐶𝑡− 1 |
𝑗= 1
𝐾𝑗>𝐾 0
𝐾𝑗≤𝑈𝑇𝐵𝑖𝑡− 1
𝐾𝑗>𝑈𝐸𝐵��𝑡− 1
𝐿𝑃𝑆 : means Lowest Put Strike, being the lowest STRIKE PRICE of the PUT OPTIONS that are comprised
in the executed Variance Replication, and being a value computed according to the following
formula:

𝐿𝑃𝑆=𝐾𝜖𝐸𝐿𝑆𝑃min
𝑡− 1
𝐾<𝐾 0
𝐾≥𝐿𝐸𝐵𝑖𝑡− 1
(𝐾)
𝐻𝐶𝑆 : means Highest Call Strike, being the highest STRIKE PRICE of the CALL OPTIONS that are comprised
in the executed Variance Replication, and being a value computed according to the following
formula:

𝐻𝐶𝑆=𝐾𝜖𝐸𝐿𝑆𝐶max
𝑡− 1
𝐾>𝐾 0
𝐾≤𝑈𝐸𝐵𝑖𝑡− 1
(𝐾)
𝟙{𝐾𝑗=𝐿𝑃��} : is equal to 1 if 𝐾𝑗 is equal to 𝐿𝑃𝑆, 0 otherwise.

𝟙{𝐾𝑗=𝐻𝐶𝑆} : is equal to 1 if 𝐾𝑗 is equal to 𝐻𝐶𝑆, 0 otherwise.

𝑃𝑢𝑡𝑀𝑖𝑡,𝐾𝑗,𝐾𝑣𝑎𝑟𝑖𝑡− 1 : means a PUT OPTION with the following attributes: EXPIRATION DATE 𝑀𝑖𝑡 , STRIKE PRICE

𝐾𝑗 and VARIANCE STRIKE 𝐾𝑣𝑎𝑟𝑖𝑡− 1

𝐶𝑎𝑙𝑙𝑀𝑖𝑡,𝐾𝑗,𝐾𝑣𝑎𝑟𝑖𝑡− 1 : means a CALL OPTION with the following attributes: EXPIRATION DATE 𝑀𝑖𝑡 , STRIKE PRICE

𝐾𝑗 and VARIANCE STRIKE 𝐾𝑣𝑎𝑟𝑖𝑡− 1

𝐼𝑛𝑑𝑒𝑥𝑡𝑇𝑅− 1 : means the TOTAL RETURN LEVEL of the INDEX on CALCULATION DAY t- 1

𝐸𝐿𝑆𝑃𝑡− 1 : the Eligible Listed Strikes Put is a subset of the ELIGIBLE LISTED STRIKES on CALCULATION DAY t-
1 , satisfying the following two conditions: the corresponding OPTION is a PUT OPTION and the STRIKE
PRICE is a multiple of 𝑆𝑡𝑒𝑝𝑉𝑅 away from 𝐾 0. The resulting subset is sorted in descending order.

𝐸𝐿𝑆𝐶𝑡− 1 : the Eligible Listed Strikes Call is a subset of the ELIGIBLE LISTED STRIKES on CALCULATION DAY t-
1 , satisfying the following two conditions: the corresponding OPTION is a CALL OPTION and the STRIKE
PRICE is a multiple of 𝑆𝑡𝑒𝑝𝑉𝑅 away from 𝐾 0. The resulting subset is sorted in ascending order.

|𝐸𝐿𝑆𝑃𝑡− 1 | : is the cardinal of the set Eligible Listed Strikes Put 𝐸𝐿𝑆𝑃𝑡− 1 as defined above.

|𝐸𝐿𝑆𝐶𝑡− 1 | : is the cardinal of the set Eligible Listed Strikes Call 𝐸𝐿𝑆𝐶𝑡− 1 as defined above.

𝑀𝑖𝑡: for each i from 1 to 5, means the i-th ELIGIBLE LISTED EXPIRATION DATE falling after CALCULATION DAY
t.

𝑉𝑆𝑉𝑅: is the VEGA SIZING of Variance Replication VR as defined in “Table 4: Old Options’
Characteristics” and “Table 5 : New Options’ Characteristics”.

𝑤𝑉𝑅: is the WEIGHT of Variance Replication VR as defined in “Table 4: Old Options’ Characteristics”
and “Table 5: New Options’ Characteristics”.

14
Version 1.1 – 30 September 2025
2.2. Selection of the ElIgible Listed Options
2.2.1. Filtering of Eligible Listed Options
On any CALCULATION DAY t, a LISTED OPTION is an “ ELIGIBLE LISTED OPTION ” if (i) its STRIKE PRICE is an ELIGIBLE
LISTED STRIKE, and (ii) its EXPIRATION DATE is an ELIGIBLE LISTED EXPIRATION DATE, as defined below:

A “ELIGIBLE LISTED EXPIRATION DATE” means an EXPIRATION DATE in respect of a LISTED OPTION
where the following condition is satisfied: There are not less than two corresponding listed
STRIKE PRICES with BID PRICES and ASK PRICES for both CALL OPTIONS and PUT OPTIONS, where the
BID PRICES are lower than or equal to the corresponding ASK PRICES.
An “ ELIGIBLE LISTED STRIKE ” means a STRIKE PRICE in respect of a LISTED OPTION where the following
condition is satisfied: The OPTION has a BID PRICE and an ASK PRICE, where the BID PRICE is lower
than or equal to the ASK PRICE.
2.2.2. Listed Options Bid/Ask Prices
On any CALCULATION DAY t in respect of the Options TWAP Window:

The “ LISTED BID PRICE ” for each available LISTED OPTION is the TWAP Bid^2 in respect of such
OPTION, as such term is defined in Section Error! Reference source not found. ., and
The “ LISTED ASK PRICE ” for each available LISTED OPTION is the TWAP Ask^3 in respect of such
OPTION, as such term is defined in Section Error! Reference source not found.
The “ LISTED MID PRICE ” for each available LISTED OPTION is the TWAP Mid^4 in respect of such
OPTION, as such term is defined in Section Error! Reference source not found..
This Section 2.2.2 is subject to the proviso that if, on any CALCULATION DAY t , the STRIKE PRICE of an
OPTION comprised in the portfolio is not an ELIGIBLE LISTED STRIKE, such OPTION’S ASK PRICE, BID PRICE or
MID PRICE (as appropriate) is computed according to Section 4.2.2 using (1) an IMPLIED VOLATILITY
determined in accordance with Section 4.2.10 using LISTED ASK PRICE, LISTED BID PRICE or LISTED MID PRICE
(as appropriate), and (2) a FORWARD and a DISCOUNT FACTOR determined in accordance with Section
4.2.6 and 4.2.8 using LISTED MID PRICES.

2.3. Selection of the Hedge Instrument
On any CALCULATION DAY t, the HEDGE INSTRUMENT is the closest to expire FUTURE CONTRACT of the Futures
Chain, unless the EXPIRATION DATE of such closest to expire FUTURE CONTRACT is less than five
CALCULATION DAYS after CALCULATION DAY t, in which case the HEDGE INSTRUMENT is the second closest to
expire FUTURE CONTRACT.

Futures Chain is the set of FUTURE CONTRACTS that are related to a specific exchange and specific
UNDERLYING ASSET.

(^2) Provided that prior to the LIVE DATE, the end of day valuation EXCHANGE BID PRICE was used, and not the TWAP Bid.
(^3) Provided that prior to the LIVE DATE, the end of day valuation EXCHANGE ASK PRICE was used, and not the TWAP Ask.
(^4) Provided that prior to the LIVE DATE, the average of the end of day valuation EXCHANGE BID PRICE and end of day valuation EXCHANGE ASK PRICE was used,
and not the TWAP Mid.

15
Version 1.1 – 30 September 2025
The “Futures Chain” is identified in the column entitled “Futures Chain RIC” in the below table:

Futures Chain RIC Exchange MIC Future Currency Price Definition
0#ES: XCME USD TWAP
Table 6: Futures Chain Parameters

2. 4. TWAP CALC UL ATION M ETH OD OLOGY
This Section Error! Reference source not found. sets out the calculation methodology for time
weighted average prices with respect to OPTIONS and the HEDGE INSTRUMENT, such prices comprising
each of the TWAP BID, TWAP ASK and TWAP MID.

The tables below define the “ Start Time ” and “ End Time ” of each of the Observations Periods and
Execution Periods that are used to compute the TWAP BID, TWAP ASK and TWAP MID.

All hours follow those of the New York Stock Exchange time zone (EST time).

Observation Period i Start Time End Time Bucket Size
𝑖= 1 9:30^ 9:^
1 second^5
𝑖= 2 10:30 10:
𝑖= 3 11:30 11:
𝑖= 4 12:30 12:
𝑖= 5 13:30^ 13:^
𝑖= 6 14:30 14:
𝑖= 7 15:30 15:
Table 7: Intraday Hedge Observation Windows

Execution Period i Start Time End Time Bucket Size
𝑖= 1 9:50 10:
1 second^5
𝑖= 2 10:50 11:
𝑖= 3 11:50 12:
��= 4 12:50 13:
𝑖= 5 13:50^ 14:^
𝑖= 6 14:50 15:
��= 7 15:50 16:
Table 8 : Intraday Hedge Execution Windows

Start Time End Time Bucket Size
15:50^6 16:00^7 1 second
Table 9: Options TWAP Window

TWAP MID, TWAP BID, TWAP ASK

“ TWAP MID ” is defined as the time weighted average mid price on a given second s over a window
of n seconds, calculated in accordance with the following formula:

(^5) Prior to the LIVE DATE, the Bucket Size was 1 minute.
(^6) On a Calculation Day that is a Half Trading Day, the Options TWAP Window Start Time shall be 12:50 EST.
(^7) On a Calculation Day that is a Half Trading Day, the Options TWAP Window End Time shall be 13:00 EST.

16
Version 1.1 – 30 September 2025
𝑇𝑊𝐴𝑃𝑠𝑛(𝑀𝑖𝑑)= 𝑇𝑊𝐴𝑃𝑠
𝑛(𝐵𝑖𝑑)+𝑇𝑊𝐴𝑃𝑠𝑛(𝐴𝑠𝑘)
2
Where:
𝑇𝑊𝐴𝑃𝑠𝑛(𝐵𝑖𝑑)= TWAP BID as defined below
𝑇𝑊𝐴𝑃𝑠𝑛(𝐴𝑠𝑘)= TWAP ASK as defined below
“ TWAP BID ” is defined as the time weighted average bid price on a given second s over a window of
n seconds, calculated in accordance with the following formula:

𝑇𝑊𝐴𝑃𝑠𝑛(𝐵𝑖𝑑)=
1
𝑛′
∑𝐵𝑖𝑑(𝑠−𝑖)
𝑛− 1
𝑖= 0
Where:
Bid(𝑡) is the prevailing EXCHANGE BID PRICE of a Valid Quote at time 𝑡 or, if no Valid Quote
is observed at this time, zero;
𝑛′: represents the number of Valid Quotes in the interval in which the average is
computed.
“Valid Quote”: An EXCHANGE BID PRICE/EXCHANGE ASK PRICE quote is deemed to be a Valid
Quote if both EXCHANGE BID PRICE and EXCHANGE ASK PRICE are non-null, with (i) the EXCHANGE
ASK PRICE being greater than or equal to the EXCHANGE BID PRICE, and (ii) the EXCHANGE BID
PRICE being above zero.
“ TWAP ASK ” is defined as the time weighted average ask price on a given second s over a window of
n seconds, calculated in accordance with the following formula:

𝑇𝑊𝐴𝑃𝑠𝑛(𝐴𝑠𝑘)=
1
𝑛′
∑𝐴𝑠𝑘(𝑠−𝑖)
𝑛− 1
𝑖= 0
Where:
Ask(𝑡) is the prevailing EXCHANGE ASK PRICE of the Valid Quote at time 𝑡 or, if no Valid
Quote is observed at this time, zero;
𝑛′: represents the number of Valid Quotes in the interval in which the average is
computed.
“Valid Quote”: An EXCHANGE BID PRICE/EXCHANGE ASK PRICE quote is deemed to be a Valid
Quote if both EXCHANGE BID PRICE and EXCHANGE ASK PRICE are non-null, with (i) the EXCHANGE
ASK PRICE being greater than or equal to the EXCHANGE BID PRICE, and (ii) the EXCHANGE BID
PRICE being above zero.
17
Version 1.1 – 30 September 2025
REBALANCE
3.1. Ordinary Rebalance
No ordinary rebalance takes place.

3.2. Extraordinary Rebalance.....................................................................................................................................
No extraordinary rebalance takes place.

18
Version 1.1 – 30 September 2025
4. Calculation of the Index
4.1. Index Formula
The “ EXCESS RETURN LEVEL ” of the INDEX 𝐼𝑛𝑑𝑒𝑥𝑡𝐸�� is calculated in accordance with the following
formula:
In relation to START DATE t 0 :
𝐼𝑛𝑑𝑒𝑥𝑡𝐸𝑅 0 = 100
On each following CALCULATION DAY t:
𝐼𝑛𝑑𝑒𝑥𝑡𝐸𝑅=𝐼𝑛𝑑𝑒𝑥𝑡𝐸𝑅− 1 +𝐼𝑛𝑑𝑒𝑥𝑡𝑇𝑅− 𝐼𝑛𝑑𝑒𝑥𝑡𝑇𝑅− 1 ×( 1 +𝑂𝑁𝑡− 1 ×
𝐴𝑐𝑡(𝑡− 1 ,𝑡)
360
)
The “ TOTAL RETURN LEVEL ” of the INDEX 𝐼𝑛𝑑𝑒𝑥𝑡𝑇𝑅 is calculated in accordance with the following
formula:
In relation to START DATE t 0 :
𝐼𝑛𝑑𝑒𝑥𝑡𝑇𝑅 0 = 100
On each following CALCULATION DAY t:
𝐼𝑛𝑑𝑒𝑥𝑡𝑇𝑅=𝑃𝑜𝑟𝑡𝑓𝑜𝑙𝑖𝑜𝑀𝑡𝑀𝑡+𝐶𝑎𝑠ℎ𝑡
Where:
𝐼𝑛𝑑𝑒𝑥𝑡𝑇𝑅 : means the TOTAL RETURN LEVEL of the INDEX on CALCULATION DAY t
𝐼𝑛𝑑𝑒𝑥𝑡𝑇𝑅− 1 : means the TOTAL RETURN LEVEL of the INDEX on CALCULATION DAY t- 1
𝐼𝑛𝑑𝑒𝑥𝑡𝐸𝑅: means the EXCESS RETURN LEVEL of the INDEX on CALCULATION DAY t
𝐼𝑛𝑑𝑒𝑥𝑡𝐸𝑅− 1 : means the EXCESS RETURN LEVEL of the INDEX on CALCULATION DAY t- 1
𝑂𝑁𝑡− 1 : Overnight rate (SOFRRATE Index, provided that prior to 2 April 2018 FEDL01 Index is used)
level on CALCULATION DAY t- 1 (or, if such a rate is not available, the immediately preceding rate)
𝐴𝑐𝑡(𝑡− 1 ,𝑡) : means the number of calendar days from, and including, CALCULATION DAY t- 1 to, but
excluding, the CALCULATION DAY t
𝑃𝑜𝑟𝑡𝑓𝑜𝑙𝑖𝑜𝑀𝑡𝑀𝑡: means the PORTFOLIO MARK-TO-MARKET in respect of CALCULATION DAY t
𝐶𝑎𝑠ℎ𝑡 : means the CASH AMOUNT in respect of CALCULATION DAY t
4.1.1. Portfolio Mark-To-Market.........................................................................................................................
In relation to CALCULATION DAY t, the PORTFOLIO MARK-TO-MARKET 𝑃𝑜��𝑡𝑓𝑜𝑙𝑖𝑜𝑀𝑡𝑀𝑡 is calculated in
accordance with the following formula:
𝑃𝑜𝑟𝑡𝑓𝑜𝑙𝑖𝑜𝑀𝑡𝑀𝑡= ∑ 𝑈𝑛𝑖𝑡𝑠𝑂×𝑀𝑖𝑑𝑡,𝑂
𝑂∈𝐶𝑂𝑃𝑡
𝑇𝑈��>𝑡 𝐴𝑁𝐷 𝑇𝐸𝑂>𝑡
19
Version 1.1 – 30 September 2025
Where:

𝐶𝑂𝑃𝑡: each OPTION 𝑂 comprising the CONTINUING OPTION PORTFOLIO in respect of CALCULATION DAY t, as
described in Section 4.1.

𝑈𝑛𝑖𝑡𝑠𝑂: the NUMBER OF UNITS in respect of OPTION 𝑂 as defined in Section 2.1.2.

𝑀𝑖𝑑𝑡,𝑂 : the MID PRICE of OPTION 𝑂 in respect of CALCULATION DAY t

𝑇𝑈𝑂 : the UNWIND DATE of OPTION 𝑂 as defined in Section 2.1.

𝑇𝐸𝑂 : the EXPIRATION DATE of OPTION 𝑂 as defined in Section 2.1.

4.1.2. Continuing Option Portfolio
In relation to CALCULATION DAY t, the CONTINUING OPTION PORTFOLIO 𝐶𝑂��𝑡 is the set comprising of each
OPTION 𝑂 that satisfies the following criteria:

TRADE DATE (𝑇𝑅𝑂) in respect of OPTION 𝑂 falls on or prior to CALCULATION DAY t
EXPIRATION DATE (𝑇𝐸𝑂) in respect of OPTION 𝑂 falls after CALCULATION DAY t
UNWIND DATE (𝑇𝑈𝑂) in respect of OPTION 𝑂 falls after CALCULATION DAY t
4.1.3. Cash Amount
The CASH AMOUNT 𝐶𝑎𝑠ℎ𝑡 is calculated in accordance with the following formula:

In relation to START DATE t 0 :
𝐶𝑎𝑠ℎ𝑡 0 = 100
On each following CALCULATION DAY t:
𝐶𝑎𝑠ℎ𝑡=𝐶𝑎𝑠ℎ𝑡− 1 ×( 1 +𝑂𝑁𝑡− 1 ×
𝐴𝐶𝑇𝑡− 1 ,𝑡
360
)− 𝑃𝑅𝑡+ 𝐸𝑉𝑡+ 𝑈𝑉𝑡+𝐷��𝑉𝑡−𝐷𝐻𝐹𝑡
Where:

𝑃𝑅𝑡: the PREMIUM PAID in respect of CALCULATION DAY t

𝐸𝑉𝑡: the EXERCISE VALUES in respect of CALCULATION DAY t

𝑈𝑉𝑡: the UNWIND VALUES in respect of CALCULATION DAY t

𝐷𝐻𝑉𝑡: the DELTA HEDGE VALUES in respect of CALCULATION DAY t

𝐷𝐻𝐹𝑡: the DELTA HEDGE FEE in respect of CALCULATION DAY t

𝑂𝑁𝑡− 1 : the Overnight rate (SORFRATE Index) level as of the CALCULATION DAY t- 1 (or, if such a rate is
not available, the immediately preceding rate)

𝐴𝐶𝑇𝑡− 1 ,��: the number of calendar days from, and including, CALCULATION DAY t-1 to, but excluding
CALCULATION DAY t

20
Version 1.1 – 30 September 2025
4.1.4. Premium Paid
In relation to CALCULATION DAY t, the PREMIUM PAID 𝑃𝑅𝑡 is calculated in accordance with the following
formula:

𝑃𝑅𝑡= ∑ 𝑃𝑂,𝑡
𝑂∈𝐶𝑂𝑃𝑡 𝐴𝑁𝐷 𝑇𝑅𝑂=𝑡
with

𝑃𝑂,𝑡={
𝑈𝑛𝑖𝑡𝑠𝑇𝑅𝑂,𝑂×𝑀𝑎𝑥( 0 ,𝐴𝑠𝑘𝑡,𝑂+ 𝑓×𝑣𝑒𝑔𝑎(𝑡,𝑂)), 𝑖𝑓 𝑈𝑛𝑖𝑡𝑠𝑇𝑅𝑂,𝑂> 0
𝑈𝑛𝑖𝑡𝑠𝑇𝑅��,𝑂×𝑀𝑎𝑥( 0 ,𝐵𝑖𝑑𝑡,𝑂 − 𝑓×𝑣𝑒𝑔𝑎(𝑡,𝑂)), 𝑖𝑓 𝑈𝑛𝑖𝑡𝑠𝑇𝑅𝑂,𝑂< 0
Where:

𝐶𝑂𝑃𝑡 : each OPTION 𝑂 comprising the CONTINUING OPTION PORTFOLIO in respect of CALCULATION DAY t, as
described in Section 4.1.2.

𝑈𝑛𝑖𝑡𝑠𝑇𝑅𝑂,𝑂 : the NUMBER OF UNITS in respect of OPTION 𝑂 as defined in Section 2.1.2.

𝑇𝑅𝑂 : the TRADE DATE of OPTION 𝑂 as defined in Section 2.1.

𝐴𝑠𝑘𝑡,𝑂 : the ASK PRICE of OPTION 𝑂 in respect of CALCULATION DAY t

𝐵𝑖𝑑𝑡,𝑂 : the BID PRICE of OPTION 𝑂 in respect of CALCULATION DAY t

𝑠𝑖𝑔𝑛(𝑥) : 1 if 𝑥 > 0 otherwise - 1

𝑓 : the FRICTION as defined in Section 2.1.

𝑉𝑒𝑔𝑎𝑡,𝑂 : the VEGA of OPTION 𝑂 in respect of CALCULATION DAY t as defined in Section 4.2.4.

4.1.5. Exercise Values..........................................................................................................................................
In relation to CALCULATION DAY t, the EXERCISE VALUES 𝐸𝑉𝑡 is calculated in accordance with the following
formula:

𝐸��𝑡= ∑ 𝑈𝑛𝑖𝑡𝑠𝑇𝑅𝑂,𝑂× 𝑃𝑎𝑦𝑜𝑓𝑓𝑡,𝑂
𝑂∈��𝑂𝑃𝑡− 1 𝐴𝑁𝐷 𝑇𝐸𝑂=𝑡
Where:

𝑈𝑛𝑖𝑡𝑠𝑇𝑅𝑂,𝑂: the NUMBER OF UNITS in respect of OPTION 𝑂 traded on TRADE DATE 𝑇𝑅𝑂

𝑃𝑎𝑦𝑜𝑓𝑓𝑡,𝑂: the PAYOUT of OPTION 𝑂 as of CALCULATION DAY t, as defined in Section 4.2.1.

4.1.6. Unwind Values
In relation to CALCULATION DAY t, the UNWIND VALUES 𝑈𝑉𝑡 is calculated in accordance with the following
formula:

21
Version 1.1 – 30 September 2025
𝑈𝑉𝑡= ∑ 𝑈𝑉𝑂,𝑡
𝑂∈𝐶𝑂𝑃𝑡 𝐴𝑁𝐷 𝑇𝑈𝑂=𝑡 ��𝑁𝐷 𝑇𝐸𝑂>𝑡
With

𝑈𝑉𝑂,𝑡={
𝑈𝑛𝑖𝑡𝑠𝑇𝑅𝑂,𝑂×𝑀𝑎𝑥( 0 ,𝐵𝑖𝑑𝑡,𝑂−𝑓×𝑣𝑒𝑔𝑎(𝑡,𝑂)), 𝑖𝑓 𝑈𝑛𝑖𝑡𝑠𝑇𝑅𝑂,𝑂> 0
𝑈𝑛𝑖𝑡𝑠𝑇𝑅𝑂,𝑂×𝑀𝑎𝑥( 0 ,𝐴𝑠𝑘𝑡,𝑂+𝑓×𝑣𝑒𝑔𝑎(𝑡,𝑂)), 𝑖𝑓 𝑈𝑛��𝑡𝑠𝑇𝑅𝑂,𝑂< 0
Where:

𝑈𝑛𝑖𝑡𝑠𝑇𝑅𝑂,𝑂: the NUMBER OF UNITS in respect of OPTION 𝑂 traded on TRADE DATE 𝑇𝑅𝑂

𝐴𝑠𝑘𝑡,𝑂 : the ASK PRICE of OPTION 𝑂 in respect of CALCULATION DAY t

𝐵𝑖𝑑𝑡,𝑂 : the BID PRICE of OPTION 𝑂 in respect of CALCULATION DAY t

𝑓 : the FRICTION as defined in Section 2.1

𝑉𝑒𝑔𝑎𝑡,𝑂 : the VEGA of OPTION 𝑂 in respect of CALCULATION DAY t

4.1.7. Delta Hedge Values
In relation to CALCULATION DAY t, the DELTA HEDGE VALUES 𝐷𝐻𝑉𝑡 is calculated in accordance with the
following formula:

𝐷𝐻𝑉𝑡= ∑ 𝑂𝐷��𝑉𝑂,𝑡
𝑂∈𝐶𝑂𝑃𝑡− 1 𝐴𝑁𝐷 𝑇𝐸𝑂≥𝑡
Where:

��𝑂𝑃𝑡 : each OPTION 𝑂 comprising the CONTINUING OPTION PORTFOLIO in respect of CALCULATION DAY t as
defined in Section 4.1.2

𝑇𝐸𝑂 : the EXPIRATION DATE of OPTION 𝑂 as defined in Section 2.1.

��𝐷𝐻𝑉𝑂,𝑡: the DELTA HEDGE VALUE of OPTION 𝑂 as of CALCULATION DAY t and is calculated according to
the following formula:

𝑂𝐷𝐻𝑉𝑂,𝑡=∑𝐼𝐷𝐻��𝑂,𝑡,𝑖
𝑀
𝑖= 1
Where:

𝑀 : is the number of Execution Periods as defined in Section 2.4.

𝐼𝐷��𝑉𝑂,𝑡,𝑖: the intraday DELTA HEDGE VALUE of OPTION 𝑂 as of CALCULATION DAY t for the i-th Execution
Period such that 1 ≤𝑖≤𝑀 and is computed according to the following formula:

𝐼𝐷𝐻𝑉𝑂,𝑡,𝑖= −𝑛𝑏𝑂,𝑡,𝑖− 1 ×(𝐹𝑢𝑡𝑡𝑒𝑥𝑒𝑐,𝑖 −𝐹𝑢𝑡𝑡��𝑥𝑒𝑐,𝑖− 1 )
Where:

22
Version 1.1 – 30 September 2025
𝐹𝑢𝑡𝑡𝑒𝑥𝑒𝑐,𝑖 : the TWAP MID of the HEDGE INSTRUMENT (as defined in Section 2.4) as of CALCULATION DAY t for
Execution Period 𝑖

𝐹𝑢��𝑡𝑒𝑥𝑒𝑐,𝑖− 1 : the TWAP MID of the HEDGE INSTRUMENT (as defined in Section 2.4) as of CALCULATION DAY t for
Execution Period 𝑖− 1 such that 1 <��≤𝑀. Otherwise, if 𝑖= 1 , then 𝐹𝑢𝑡𝑡𝑒𝑥𝑒𝑐, 0 =𝐹𝑢𝑡𝑡𝑒𝑥𝑒𝑐− 1 ,𝑀

𝑛𝑏𝑂,𝑡,𝑖− 1 : the NUMBER OF UNITS] of the HEDGE INSTRUMENT held on CALCULATION DAY t right before the i-
th Execution Period and defined as the following:

For 2 ≤𝑖≤��:

𝑛𝑏𝑂,𝑡,𝑖− 1 =𝑈𝑛𝑖𝑡𝑠𝑇𝑅𝑂,𝑂×𝑑𝑒𝑙𝑡��(𝑡− 1 ,𝑂,𝐾𝑣𝑎𝑟𝑂,𝐹𝑤𝑑𝑡− 1 ,𝑇𝐸𝑂×
𝐹𝑢𝑡𝑡𝑜𝑏𝑠,𝑖− 1
𝐹𝑢𝑡𝑡𝑒𝑥𝑒𝑐− 1 ,𝑀
)×
𝐹𝑤𝑑𝑡− 1 ,𝑇𝐸𝑂
𝐹𝑢𝑡𝑡𝑒𝑥𝑒𝑐− 1 ,𝑀
For 𝑖= 1 :

��𝑏𝑂,𝑡, 0 =𝑛𝑏𝑂,𝑡− 1 ,𝑀=𝑈𝑛𝑖𝑡𝑠𝑇𝑅𝑂,𝑂×𝑑𝑒𝑙𝑡𝑎(𝑡− 1 ,𝑂,𝐾𝑣𝑎𝑟𝑂,𝐹𝑤𝑑𝑡− 2 ,𝑇𝐸��×
𝐹𝑢𝑡𝑡𝑜𝑏𝑠− 1 ,𝑀
𝐹𝑢𝑡𝑡𝑒𝑥𝑒𝑐− 2 ,𝑀
)×
𝐹𝑤𝑑𝑡− 2 ,𝑇𝐸𝑂
𝐹𝑢𝑡𝑡𝑒𝑥𝑒𝑐− 2 ,𝑀
Where:

𝐹𝑤𝑑𝑡− 1 ,𝑇𝐸𝑂 : the FORWARD of EXPIRATION DATE 𝑇𝐸𝑂 computed on CALCULATION DAY t- 1 according to
Section 4.2.9.

𝐾𝑣𝑎𝑟𝑂 : the VARIANCE STRIKE associated with OPTION 𝑂 as defined in Section 2.2.

𝐹𝑢𝑡𝑡𝑜𝑏𝑠,𝑖 : the TWAP MID of the HEDGE INSTRUMENT (as defined in Section 2.3) as of CALCULATION DAY t for
Observation Period 𝑖 such that 1 ≤𝑖≤𝑀.

𝐾𝑣𝑎𝑟 : the VARIANCE STRIKE associated with OPTION 𝑂.

4.1.8. Delta Hedge Fee
In relation to CALCULATION DAY t, the DELTA HEDGE VALUES 𝐷𝐻𝑉𝑡 is calculated in accordance with the
following formula:

𝐷𝐻𝐹𝑡=𝐼𝐷𝐻𝐹𝑡− 1 ,𝑀+∑𝐼𝐷𝐻𝐹𝑡,𝑖
𝑀− 1
𝑖= 1
Where:

𝑀 : is the number of Execution Periods as defined in Section 2. 4.

𝐼𝐷𝐻𝐹𝑡,𝑖: the intraday DELTA HEDGE FEE as of CALCULATION DAY t for the i-th Execution Period and is
computed according to the following formula:

For 1 ≤𝑖≤𝑀− 1 :

𝐷𝐻𝐹𝑡,𝑖=𝐹𝑢𝑡𝑡𝑒𝑥��𝑐,𝑖 ×𝑑ℎ𝑓×𝑎𝑏𝑠( ∑ 𝑛𝑏𝑂,𝑡,𝑖−𝑛𝑏𝑂,𝑡,𝑖− 1
𝑂𝜖 𝐶𝑂𝑃𝑡− 1
)
𝐷𝐻𝐹𝑡, 1 =𝐹𝑢𝑡𝑡𝑒𝑥𝑒��, 1 ×𝑑ℎ𝑓×𝑎𝑏𝑠( ∑ 𝑛𝑏𝑂,𝑡, 1
𝑂𝜖 𝐶𝑂𝑃𝑡− 1
)+𝐹𝑢𝑡𝑡𝑒𝑥𝑒𝑐, 1 ,𝐹𝑟𝑜𝑛𝑡^ 𝑚𝑜𝑛𝑡ℎ×𝑑ℎ𝑓×𝑎𝑏𝑠( ∑ −𝑛𝑏𝑂,𝑡, 0
𝑂𝜖 𝐶𝑂𝑃𝑡− 1
)^ 𝑤ℎ𝑒𝑛^ 𝑡=𝑅𝑜𝑙𝑙𝑖𝑛𝑔^ 𝐹𝑢𝑡𝑢𝑟𝑒^ 𝐷𝑎𝑡𝑒^
23
Version 1.1 – 30 September 2025
For 𝑖=𝑀:

𝐷𝐻𝐹𝑡,𝑀=𝐹𝑢𝑡𝑡𝑒𝑥𝑒𝑐,𝑀 ×𝑑ℎ𝑓
×𝑎𝑏𝑠
(
∑ (𝑛𝑏𝑂,𝑡,𝑀−𝑛𝑏𝑂,𝑡,𝑀− 1 )
𝑂𝜖 𝐶𝑂𝑃𝑡− 1
𝐴𝑁𝐷 𝑇𝐸𝑜>𝑡
𝐴𝑁𝐷 𝑇𝑈𝑜>𝑡
+ ∑ −𝑛𝑏𝑂,𝑡,𝑀− 1
𝑂𝜖 𝐶𝑂𝑃𝑡− 1
𝐴𝑁𝐷 (𝑇𝐸𝑜=𝑡 𝑂𝑅 𝑇𝑈𝑜=𝑡)
+ ∑ 𝑛𝑏𝑂,𝑡,𝑀
𝑂𝜖 𝐶𝑂𝑃𝑡
𝐴𝑁𝐷 𝑇𝑅𝑜=𝑡 )
Where:

𝑑ℎ𝑓 : is the DELTA HEDGE FEE as defined in Section 2.1.

𝐹𝑢𝑡𝑡𝑒𝑥𝑒𝑐,𝑖 : the TWAP MID of the HEDGE INSTRUMENT (as defined in Section 2.4) as of CALCULATION DAY t on
the i-th Execution Period

𝑛𝑏𝑂,𝑡,𝑖− 1 : the NUMBER OF UNITS of the HEDGE INSTRUMENT held on CALCULATION DAY t right before the i-
th Execution Period as defined in Section 4.1.8.

𝐶𝑂𝑃𝑡 : each OPTION 𝑂 comprising the CONTINUING OPTION PORTFOLIO in respect of CALCULATION DAY t as
defined in Section 4.1.2.

��𝐸𝑂 : the EXPIRATION DATE of OPTION 𝑂 as defined in Section 2.1.

𝑇𝑅𝑂 : the TRADE DATE of OPTION 𝑂 as defined in Section 2.1.

𝑇𝑈𝑂 : the UNWIND DATE of OPTION 𝑂 as defined in Section 2.1.

𝑅𝑜𝑙𝑙𝑖𝑛𝑔 𝐹𝑢𝑡𝑢𝑟𝑒 𝐷𝑎𝑡𝑒 : as defined in Section 2.1.

𝐹𝑢𝑡𝑡𝑒𝑥𝑒𝑐,𝑖 ,𝐹𝑟𝑜𝑛𝑡^ 𝑚𝑜𝑛𝑡ℎ : the TWAP MID of the closest to expire FUTURE CONTRACT of the Futures Chain (as
defined in Section 2.3) as of CALCULATION DAY t on the i-th Execution Period

4.2. Option Pricing Methodology
4.2.1. Payoff
In relation to OPTION 𝑂, the PAYOUT 𝑃𝑎𝑦𝑜𝑓𝑓𝑡,𝑂 is calculated in accordance with the following formula:

𝑃𝑎𝑦𝑜𝑓𝑓𝑂,𝑡=𝑚𝑎��( 0 ,𝐶𝑃×( 𝑈𝑆𝐼𝑡−𝐾𝑂))
Where:

𝐶𝑃 : whether the OPTION O is OPTION TYPE Call (𝐶𝑃= 1 ) or OPTION TYPE Put (𝐶𝑃=− 1 )

𝑀𝑎𝑥: means the MAXIMUM FUNCTION

𝑈𝑆𝐼𝑡: the UNDERLYING CLOSING INDEX LEVEL as of CALCULATION DAY t

𝐾𝑂 : the STRIKE PRICE of OPTION 𝑂

24
Version 1.1 – 30 September 2025
4.2.2. Premium
In relation to OPTION 𝑂, the PREMIUM 𝑃𝑋𝑡,𝑂 as of CALCULATION DAY t is calculated in accordance with
the following formula:

𝑃𝑋𝑡,𝑂=𝑃𝑋(𝑡,𝐶𝑃,𝐹𝑤𝑑𝑡,𝑇𝐸𝑂,𝐷𝐹𝑡,𝑇��𝑂,𝑇𝐸𝑂,𝐾𝑂,𝜎𝑡,𝐾𝑂,𝑇𝐸𝑂)
=𝐷𝐹𝑡,𝑇𝐸𝑂×��𝑃
× (𝐹𝑤𝑑𝑡,𝑇𝐸𝑂×𝑁(𝐶𝑃×𝑑 1 ,𝐾𝑂,𝑇𝐸𝑂,��(𝜎𝑡,𝐾𝑂,𝑇𝐸𝑂))−𝐾𝑂×𝑁(𝐶𝑃×𝑑 2 ,𝐾𝑂,𝑇𝐸��,𝑡(𝜎𝑡,𝐾𝑂,𝑇𝐸𝑂)))
Where:
𝑑 1 ,𝐾,𝑇𝐸,𝑡(��)=
log(𝐹𝑤𝑑𝐾𝑡,𝑇𝐸)+𝜎
2
2 ×𝐷𝐶𝐹𝑡,𝑇𝐸
𝜎×√𝐷𝐶𝐹𝑡,𝑇𝐸
and
𝑑 2 ,𝐾,𝑇𝐸,𝑡(𝜎)=𝑑 1 ,𝐾,��𝐸,𝑡(𝜎)−𝜎×√𝐷𝐶𝐹𝑡,𝑇𝐸
With:

𝐹𝑤𝑑𝑡,𝑇𝐸��: the FORWARD in relation to CALCULATION DAY t and EXPIRATION DATE 𝑇𝐸𝑂 as calculated in
accordance with Section 4.2.8.

𝐷𝐹𝑡,𝑇𝐸𝑂: the DISCOUNT FACTOR in relation to CALCULATION DAY t and EXPIRATION DATE 𝑇𝐸𝑂 as calculated
in accordance with Section 4.2.6 6.

𝜎𝑡,𝐾𝑂,𝑇𝐸𝑂: the IMPLIED VOLATILITY 𝜎 as of CALCULATION DAY t in relation to STRIKE PRICE 𝐾𝑂 of OPTION 𝑂 and
EXPIRATION DATE 𝑇𝐸𝑂 as calculated in accordance with Section 4.2.8 10.

𝐷𝐶𝐹𝑡,𝑇𝐸: the DAY COUNT FRACTION in respect to EXPIRATION DATE 𝑇𝐸 as of CALCULATION DAY t as defined
in Section 4.2.6.

𝐾𝑂 : the STRIKE PRICE of OPTION 𝑂

𝑇𝐸𝑂: the EXPIRATION DATE of OPTION 𝑂

𝑁(𝑥): CUMULATIVE DISTRIBUTION FUNCTION of the Standard Normal Distribution, being a value computed
according to the following formula:

𝑁(𝑥)=
1
√^2 𝜋
∫ 𝑒−
𝑢^2
(^2) 𝑑𝑢
𝑥
−∞
log(.): The NATURAL LOGARITHM FUNCTION

4.2.3. Eligible Listed Option Implied Volatility
The ELIGIBLE LISTED OPTION IMPLIED VOLATILITY in relation to an ELIGIBLE LISTED OPTION 𝑂 with STRIKE PRICE ��
and EXPIRATION DATE 𝑇𝐸 on any CALCULATION DAY t is calculated as the IMPLIED VOLATILITY 𝜎 for which
the PREMIUM for such OPTION matches the price of the ELIGIBLE LISTED OPTION (LISTED BID PRICE, LISTED ASK
PRICE, or LISTED MID PRICE):

𝑃𝑟𝑖𝑐��𝑡𝑇𝐸,𝑂,𝐾=𝑃𝑋𝑡,𝑂 =𝑃𝑋(𝐶𝑃,𝐹𝑤𝑑𝑡,𝑇𝐸,��𝐹𝑡,𝑇𝐸,𝐾,𝑡,𝑇𝐸,𝜎)
With:

25
Version 1.1 – 30 September 2025
𝑃𝑟𝑖𝑐𝑒𝑡𝑇𝐸,𝑂,𝐾: is either the LISTED BID PRICE, LISTED ASK PRICE or LISTED MID PRICE in respect of CALCULATION
DAY t of the ELIGIBLE LISTED OPTION 𝑂 expiring on EXPIRATION DATE 𝑇𝐸 with a STRIKE PRICE 𝐾

𝑃𝑋𝑡,𝑂: The PREMIUM of OPTION 𝑂 as of CALCULATION DAY t as determined in accordance with Section
4.2.2.

𝐶𝑃: The OPTION TYPE of ELIGIBLE LISTED OPTION 𝑂 expiring on EXPIRATION DATE 𝑇𝐸 with a STRIKE PRICE 𝐾

𝐹𝑤𝑑𝑡,𝑇𝐸: the FORWARD in relation to CALCULATION DAY t and EXPIRATION DATE 𝑇𝐸

𝐷𝐹𝑡,𝑇𝐸: the DISCOUNT FACTOR in relation to CALCULATION DAY t and EXPIRATION DATE 𝑇𝐸

4.2.4. Option Greeks Calculation
The DELTA, VEGA, GAMMA, and THETA of any OPTION 𝑂 are computed in accordance with the following
formulas:

The DELTA 𝐷𝑒𝑙𝑡𝑎𝑡,𝑂 of OPTION 𝑂 as of CALCULATION DAY t is calculated as follows:

𝐷𝑒𝑙𝑡𝑎𝑡,𝑂=𝐷𝑒𝑙𝑡𝑎(𝑡,𝐶𝑃,𝐹𝑤𝑑𝑡,𝑇𝐸𝑂,𝐷𝐹𝑡,𝑇𝐸𝑂,𝑇𝐸𝑂,𝐾𝑂,𝜎𝑡,𝐾𝑂,𝑇𝐸𝑂)
=𝐷𝐹𝑡,𝑇𝐸𝑂 ×𝐶𝑃×𝑁(𝐶𝑃×𝑑 1 ,𝑂,𝑡(𝜎𝑡,𝐾,𝑇𝐸𝑂))
The VEGA 𝑉𝑒𝑔𝑎𝑡,𝑂 of OPTION 𝑂 as of CALCULATION DAY t is calculated as follows:

��𝑒𝑔𝑎𝑡,𝑂=𝑉𝑒𝑔𝑎(𝑡,𝐶𝑃,𝐹𝑤𝑑𝑡,𝑇𝐸𝑂,𝐷𝐹𝑡,𝑇𝐸𝑂,𝑇𝐸𝑂,𝐾𝑂,𝜎𝑡,𝐾𝑂,𝑇𝐸𝑂)
=𝐷𝐹𝑡,𝑇𝐸𝑂 ×𝐹𝑤𝑑𝑡,𝑇𝐸𝑂×𝑁′(𝑑 1 ,𝑂,𝑡(𝜎𝑡,𝐾𝑂,𝑇𝐸𝑂))×√𝐷𝐶𝐹𝑡,𝑇𝐸𝑂
The GAMMA 𝐺𝑎𝑚𝑚𝑎𝑡,𝑂 of OPTION 𝑂 as of CALCULATION DAY t is calculated as follows:

𝐺𝑎𝑚𝑚𝑎𝑡,𝑂=𝐺𝑎𝑚𝑚𝑎(𝑡,𝐶𝑃,��𝑤𝑑𝑡,𝑇𝐸𝑂,𝐷𝐹𝑡,𝑇𝐸𝑂,𝑇𝐸𝑂,𝐾𝑂,𝜎𝑡,𝐾𝑂,𝑇𝐸𝑂)
=
𝑁′(𝑑 1 ,𝑂,𝑡(𝜎𝑡,𝐾𝑂,𝑇𝐸𝑂))×𝐷𝐹𝑡,𝑇𝐸𝑂
𝐹𝑤𝑑𝑡,𝑇𝐸𝑂 × 𝜎𝑡,𝐾𝑂,𝑇𝐸𝑂 ×√𝐷��𝐹𝑡,𝑇𝐸𝑂
The THETA 𝑇ℎ𝑒𝑡𝑎𝑡,𝑂 of OPTION 𝑂 as of CALCULATION DAY t is calculated as follows:

𝑇ℎ𝑒𝑡𝑎𝑡,𝑂=𝑇ℎ𝑒𝑡𝑎(𝑡,𝐶𝑃,𝐹𝑤𝑑𝑡,𝑇𝐸𝑂,𝐷𝐹𝑡,𝑇𝐸𝑂,𝑇𝐸𝑂,𝐾𝑂,𝜎𝑡,𝐾𝑂,𝑇𝐸𝑂)
=−
𝑁′(𝑑 1 ,𝑂,𝑡(𝜎𝑡,𝐾𝑂,𝑇𝐸𝑂))×𝐷𝐹𝑡,𝑇𝐸𝑂× 𝐹𝑤𝑑𝑡,𝑇𝐸𝑂×𝜎𝑡,𝐾𝑂,𝑇𝐸𝑂
2 ×√𝐷𝐶𝐹𝑡,𝑇𝐸𝑂
+
ln( 𝐷𝐹𝑡,𝑇𝐸𝑂)×𝐶𝑃
𝐷𝐶𝐹𝑡,𝑇𝐸𝑂
×[𝐾𝑂×𝐷𝐹𝑡,𝑇𝐸𝑂×𝑁(𝐶𝑃×𝑑 2 ,𝐾𝑂,𝑇𝐸𝑂,𝑡(𝜎𝑡,𝐾𝑂,𝑇𝐸𝑂))
−𝐹𝑤𝑑𝑡,𝑇𝐸𝑂×𝐷𝐹𝑡,𝑇𝐸𝑂× 𝑁(𝐶��×𝑑 1 ,𝐾𝑂,𝑇𝐸𝑂,𝑡(𝜎𝑡,𝐾𝑂,𝑇𝐸𝑂))]
26
Version 1.1 – 30 September 2025
Where:

𝑑 1 ,𝑂,𝑡(𝐾)=
log(
𝐹𝑤𝑑𝑡,𝑇𝐸𝑂
𝐾 )+
𝜎𝑡,𝐾,𝑇𝐸𝑂^2
2 ×𝐷𝐶𝐹𝑡,𝑇𝐸𝑂
𝜎𝑡,𝐾,𝑇𝐸𝑂×√𝐷𝐶𝐹𝑡,𝑇𝐸𝑂
With:

𝐶𝑃 : whether the OPTION 𝑂 is OPTION TYPE Call (𝐶𝑃= 1 ) or OPTION TYPE Put (𝐶𝑃=− 1 )

𝐷��𝑡,𝑇𝐸𝑂: the DISCOUNT FACTOR in respect to EXPIRATION DATE 𝑇𝐸𝑂 of OPTION 𝑂 as of CALCULATION DAY t

𝐹𝑤𝑑𝑡,𝑇𝐸𝑂: FORWARD in respect to EXPIRATION DATE 𝑇𝐸𝑂 of OPTION 𝑂 as of CALCULATION DAY t

𝐾: The STRIKE PRICE of OPTION 𝑂

𝐷𝐶𝐹𝑡,𝑇𝐸𝑂: The DAY COUNT FRACTION in respect to EXPIRATION DATE 𝑇𝐸𝑂 of OPTION 𝑂 as of CALCULATION DAY
t as defined in Section 4.2.6.

𝑁(𝑥): CUMULATIVE DISTRIBUTION FUNCTION of the Standard Normal Distribution, being a value computed
according to the following formula:

𝑁(𝑥)=
1
√ 2 𝜋
∫ 𝑒−
𝑢^2
(^2) 𝑑𝑢
𝑥
−∞
log(.): The NATURAL LOGARITHM FUNCTION
𝜎𝑡,𝐾,𝑇𝐸𝑂: the IMPLIED VOLATILITY as of CALCULATION DAY t in relation to STRIKE PRICE 𝐾 as of EXPIRATION DATE
𝑇𝐸𝑂 of OPTION 𝑂
N′(𝑥): the density function of the Standard Normal Distribution, being a value computed according
to the following formula:

𝑁′(𝑥)=
𝑒−
𝑥^2
2
√^2 𝜋
exp(.): EXPONENTIAL FUNCTION to the Basis of Euler’s number 𝑒.

4.2.5. Day Count Fraction
The DAY COUNT FRACTION in respect of EXPIRATION DATE 𝑇𝐸 as of CALCULATION DAY t is (i) the number of
CALCULATION DAYS from (and including) CALCULATION DAY t to (but excluding) EXPIRATION DATE 𝑇𝐸 divided
by (ii) 252.

4.2.6. Discount Factor
In relation to CALCULATION DAY t and EXPIRATION DATE 𝑇𝐸, the DISCOUNT FACTOR 𝐷𝐹𝑡,𝑇𝐸 is calculated as
follows:

��𝐹𝑡,𝑇𝐸=exp(log(𝐷𝐹𝑡,𝑇 1 )+
𝐷𝐶𝑇 1 ,𝑇𝐸×(log(𝐷𝐹𝑡,𝑇 2 )−log(𝐷𝐹𝑡,𝑇 1 ))
𝐷𝐶𝑇 1 ,𝑇 2
)
27
Version 1.1 – 30 September 2025
With:

𝑇 1 : means the ELIGIBLE LISTED EXPIRATION DATE 𝑇 1 selected in accordance with Section 4.2.7.

𝑇 2 : means the ELIGIBLE LISTED EXPIRATION DATE 𝑇 2 selected in accordance with Section 4.2.7.

𝐷𝐹𝑡,�� 1 : the DISCOUNT FACTOR in relation to CALCULATION DAY t and ELIGIBLE LISTED EXPIRATION DATE 𝑇 1
calculated in accordance with Section 4.2.9. If 𝑇 1 =𝑡, then the DISCOUNT FACTOR in relation to
CALCULATION DAY t and ELIGIBLE LISTED EXPIRATION DATE 𝑇 1 is 1.

𝐷𝐹𝑡,𝑇 2 : the DISCOUNT FACTOR in relation to CALCULATION DAY t and ELIGIBLE LISTED EXPIRATION DATE 𝑇 2
calculated in accordance with Section 4.2.9 9.

𝐷𝐶𝑇 1 ,𝑇𝐸: means the NUMBER OF CALENDAR DAYS in the period commencing on (and including) ELIGIBLE
LISTED EXPIRATION DATE 𝑇 1 and ending on (but excluding) EXPIRATION DATE 𝑇��.

𝐷𝐶𝑇 1 ,𝑇 2 : means the NUMBER OF CALENDAR DAYS in the period commencing on (and including) ELIGIBLE
LISTED EXPIRATION DATE 𝑇 1 and ending on (but excluding) ELIGIBLE LISTED EXPIRATION DATE 𝑇 2_._

log(.): The NATURAL LOGARITHM FUNCTION.

exp(.): EXPONENTIAL FUNCTION to the Basis of Euler’s number 𝑒.

4.2.7. Maturity Selection
In relation to CALCULATION DAY t and EXPIRATION DATE 𝑇𝐸, two EXPIRATION DATES 𝑇 1 , 𝑇 2 are selected with
regards to 𝑇𝐸 following the below methodology:

Where EXPIRATION DATE 𝑇𝐸 is lower than any EXPIRATION DATE within the set of ELIGIBLE LISTED
EXPIRATION DATES, 𝑇 1 =𝑡 and 𝑇 2 is the shortest ELIGIBLE LISTED EXPIRATION DATE in respect of
CALCULATION DAY t.
Where EXPIRATION DATE 𝑇𝐸 is strictly greater than any EXPIRATION DATE within the set of ELIGIBLE
LISTED EXPIRATION DATES, 𝑇 1 =𝑇 2 = 𝑇𝐸.
Otherwise, (i) 𝑇 1 is the furthest ELIGIBLE LISTED EXPIRATION DATE in respect of CALCULATION DAY t
that is less than or equal to 𝑇𝐸, and (ii) �� 2 is the shortest ELIGIBLE LISTED EXPIRATION DATE in
respect of CALCULATION DAY t that is greater than or equal to 𝑇𝐸.
4.2.8. Forward
In relation to CALCULATION DAY t and EXPIRATION DATE 𝑇𝐸, the FORWARD 𝐹𝑤𝑑𝑡,𝑇𝐸 is calculated as follows:

𝐹𝑤𝑑𝑡,𝑇𝐸=exp(log(𝐹𝑤𝑑𝑡,𝑇 1 )+
𝐷𝐶𝑇 1 ,��𝐸×(log(𝐹𝑤𝑑𝑡,𝑇 2 )−log(𝐹𝑤𝑑𝑡,𝑇 1 ))
𝐷𝐶𝑇 1 ,𝑇 2
)
With:

𝑇 1 : means the ELIGIBLE LISTED EXPIRATION DATE 𝑇 1 selected in accordance with Section 4.2.7.

𝑇 2 : means the ELIGIBLE LISTED EXPIRATION DATE 𝑇 2 selected in accordance with Section 4.2.7.

28
Version 1.1 – 30 September 2025
𝐹𝑤𝑑𝑡,𝑇 1 : the FORWARD in relation to CALCULATION DAY t and EXPIRATION DATE 𝑇 1 calculated in
accordance with Section 4.2.9. If 𝑇 1 =𝑡, then the FORWARD in relation to CALCULATION DAY t and
EXPIRATION DATE 𝑇 1 is the UNDERLYING CLOSING INDEX LEVEL as of CALCULATION DAY t

𝐹𝑤𝑑𝑡,𝑇 2 : the FORWARD in relation to CALCULATION DAY t and EXPIRATION DATE 𝑇 2 calculated in
accordance with Section 4.2.9.

𝐷𝐶𝑇 1 ,𝑇 2 : means the NUMBER OF CALENDAR DAYS in the period commencing on (and including) ELIGIBLE
LISTED EXPIRATION DATE 𝑇 1 and ending on (but excluding) ELIGIBLE LISTED EXPIRATION DATE 𝑇 2

log(.): The NATURAL LOGARITHM FUNCTION

4.2.9. Discount Factor and Forward for an Eligible Listed Expiration Date
In relation to CALCULATION DAY t, for an EXPIRATION DATE of an ELIGIBLE LISTED OPTION, the DISCOUNT FACTOR
and FORWARD for that EXPIRATION DATE shall be calculated in accordance with the following
methodology:

Two STRIKE PRICES are selected, 𝐾𝑖,𝑎 and 𝐾𝑖,𝑏 with the closest CALL OPTION and PUT OPTION prices:

𝐾𝑖,𝑎^8 =𝐾𝑖,𝑗 / 𝑗=𝑎𝑟𝑔𝑚𝑖𝑛𝑗= 1 ,...,𝑛𝑖(|𝑐𝑎𝑙𝑙𝑖,𝑗−𝑝𝑢𝑡𝑖,𝑗|)
𝐾𝑖,𝑏^9 =𝐾𝑖,𝑗 / 𝑗=𝑎𝑟𝑔𝑚𝑖𝑛𝑗= 1 ,...,𝑛𝑖;𝑗≠𝑎(|𝑐𝑎𝑙𝑙𝑖,𝑗−��𝑢𝑡𝑖,𝑗|)
By fitting the Call-Put parity for those two STRIKE PRICES, the following applies:

𝐷��𝑖=
(𝑐𝑎𝑙𝑙𝑖,𝑎−𝑝𝑢𝑡𝑖,𝑎)−(𝑐𝑎𝑙𝑙𝑖,𝑏−𝑝𝑢𝑡𝑖,𝑏)
𝐾𝑖,𝑏−𝐾𝑖,𝑎
𝐹𝑖=
𝑐𝑎𝑙𝑙𝑖,𝑎−𝑝��𝑡𝑖,𝑎
𝐷𝐹𝑖
+𝐾𝑖,𝑎
4.2.10. Implied Volatility
In relation to CALCULATION DAY t, STRIKE PRICE �� and EXPIRATION DATE 𝑇𝐸, the IMPLIED VOLATILITY 𝜎𝑡,𝐾,𝑇𝐸 is
calculated based on the following methodology:

In order to calculate the IMPLIED VOLATILITY, up to four LISTED OPTIONS are required.

In relation to CALCULATION DAY t and EXPIRATION DATE 𝑇𝐸, two EXPIRATION DATES 𝑇 1 , 𝑇 2 are selected in
accordance with Section 4.2.7.

The DISCOUNT FACTOR and FORWARD for the two selected EXPIRATION DATES are calculated in accordance
with Section 4.2.9.

With respect to each selected ELIGIBLE LISTED EXPIRATION DATE 𝑇𝑖, two STRIKE PRICES 𝐾 1 , and 𝐾 2 are
selected using the following criteria:

(^8) When several STRIKE PRICES satisfy the condition, the lowest STRIKE PRICE is chosen.
(^9) When several STRIKE PRICES satisfy the condition, the lowest STRIKE PRICE is chosen.

29
Version 1.1 – 30 September 2025
Where STRIKE PRICE 𝐾 is strictly lower than the lowest STRIKE PRICE of ELIGIBLE LISTED OPTION in
respect of CALCULATION DAY t and EXPIRATION DATE 𝑇𝑖 , 𝐾 2 = 𝐾 1 , where 𝐾 1 is lowest STRIKE PRICE
of ELIGIBLE LISTED OPTION in respect of CALCULATION DAY t and EXPIRATION DATE 𝑇𝑖
Where STRIKE PRICE 𝐾 is strictly higher than the highest STRIKE PRICE of ELIGIBLE LISTED OPTION in
respect of CALCULATION DAY t and EXPIRATION DATE 𝑇𝑖 , 𝐾 1 = 𝐾 2 , where 𝐾 2 is the highest STRIKE
PRICE of ELIGIBLE LISTED OPTION in respect of CALCULATION DAY t and EXPIRATION DATE 𝑇𝑖
Otherwise, (i) 𝐾 1 is the highest STRIKE PRICE of ELIGIBLE LISTED OPTION in respect of CALCULATION
DAY t and EXPIRATION DATE 𝑇𝑖that is less than or equal to STRIKE PRICE 𝐾, and (ii) 𝐾 2 is the lowest
STRIKE PRICE of ELIGIBLE LISTED OPTION in respect of CALCULATION DAY t and EXPIRATION DATE 𝑇�� that
is higher than or equal to STRIKE PRICE 𝐾
The four selected ELIGIBLE LISTED OPTIONS are set to be of OPTION TYPE Put.

Once the DISCOUNT FACTOR, FORWARD, EXPIRATION DATE and STRIKE PRICE are determined for the four
selected ELIGIBLE LISTED OPTIONS, the IMPLIED VOLATILITY of each such OPTION is determined in accordance
with Section 4.2.3, namely:

𝜎𝑡,𝐾 1 ,𝑇 1 , 𝜎𝑡,𝐾 2 ,𝑇 1 , 𝜎𝑡,𝐾 1 ,𝑇 2 , 𝜎𝑡,𝐾 2 ,𝑇 2.

The IMPLIED VOLATILITY for the ELIGIBLE LISTED OPTION with STRIKE PRICE 𝐾 and for the two selected
ELIGIBLE LISTED EXPIRATION DATE 𝑇 1 , 𝑇 2 is thus interpolated as follows:

𝜎𝑡,𝐾,𝑇 1 = {𝜎𝑡,𝐾^1 ,𝑇^1 +
(𝐾−𝐾 1 )×(𝜎𝑡,𝐾 2 ,𝑇 1 −𝜎𝑡,𝐾 1 ,𝑇 1 )
(𝐾 2 −𝐾 1 )
𝑖𝑓 𝐾 1 ≠ 𝐾 2
𝜎𝑡,𝐾 1 ,𝑇 1 𝑜𝑡ℎ𝑒𝑟𝑤𝑖𝑠𝑒
𝜎𝑡,𝐾,𝑇 2 = {𝜎𝑡,𝐾^1 ,𝑇^2 +
(𝐾−𝐾 1 )×(��𝑡,𝐾 2 ,𝑇 2 −𝜎𝑡,𝐾 1 ,𝑇 2 )
(𝐾 2 −𝐾 1 )^ ��𝑓^ 𝐾^1 ≠^ 𝐾^2
𝜎𝑡,𝐾 1 ,𝑇 2 𝑜𝑡ℎ𝑒𝑟𝑤𝑖𝑠��
Finally, the IMPLIED VOLATILITY 𝜎𝑡,𝐾,𝑇𝐸 in relation to CALCULATION DAY t, STRIKE PRICE 𝐾 and EXPIRATION DATE
𝑇𝐸 is calculated as follows:

��𝑡,𝐾,𝑇𝐸=√
1
𝐷𝐶𝑡,𝑇𝐸^ ×𝑀𝑎𝑥(^0 ,(𝜎𝑡,𝐾,𝑇^1 )
(^2) × 𝐷𝐶𝑡,𝑇
1 +^
𝐷𝐶𝑇 1 ,𝑇𝐸 × [(𝜎𝑡,𝐾,𝑇 2 )^2 × 𝐷𝐶𝑡,𝑇 2 − (𝜎𝑡,𝐾,𝑇 1 )^2 × 𝐷𝐶𝑡,𝑇 1 ]
𝐷𝐶𝑇 1 ,𝑇 2 )^ 𝑖𝑓^ 𝑇^1 ≠𝑇^2
𝜎𝑡,𝐾,𝑇 1 𝑜𝑡ℎ𝑒𝑟𝑤𝑖𝑠𝑒
With:
𝜎𝑡,𝐾,𝑇 1 : means the IMPLIED VOLATILITY in respect of Calculation Day t with EXPIRATION DATE 𝑇 1 being an
ELIGIBLE LISTED EXPIRATION DATE
𝜎𝑡,𝐾,𝑇 2 : means the IMPLIED VOLATILITY in respect of CALCULATION DAY t with EXPIRATION DATE 𝑇 2 being an
ELIGIBLE LISTED EXPIRATION DATE
𝐷𝐶𝑡,𝑇 1 : means the number of CALCULATION DAYS in the period commencing on (and including)
CALCULATION DAY t and ending on (but excluding) ELIGIBLE LISTED EXPIRATION DATE 𝑇 1
𝐷𝐶𝑡,𝑇 2 : means the number of CALCULATION DAYS in the period commencing on (and including)
CALCULATION DAY t and ending on (but excluding) ELIGIBLE LISTED EXPIRATION DATE 𝑇 2

30
Version 1.1 – 30 September 2025
𝐷𝐶𝑇 1 ,𝑇𝐸: means the number of CALCULATION DAYS in the period commencing on (and including) ELIGIBLE
LISTED EXPIRATION DATE 𝑇 1 and ending on (but excluding) EXPIRATION DATE ��𝐸

𝐷𝐶𝑇 2 ,𝑇𝐸: means the number of CALCULATION DAYS in the period commencing on (and including) ELIGIBLE
LISTED EXPIRATION DATE 𝑇 2 and ending on (but excluding) EXPIRATION DATE 𝑇𝐸

4.3. Accuracy..............................................................................................................................................................
The level of the INDEX will be rounded to 4 decimal places.

4.4. Recalculation
The INDEX ADMINISTRATOR makes the greatest possible efforts to accurately calculate and maintain the
INDEX. However, errors in the determination process may occur from time to time for a variety of
reasons (internal or external) and therefore cannot be completely ruled out in respect of any INDEX.
The INDEX ADMINISTRATOR endeavors to correct all errors that have been identified within a reasonable
period of time. The understanding of “a reasonable period of time” as well as the general measures
to be taken generally depend on the underlying and is specified in the SOLACTIVE Correction Policy,
which is incorporated by reference and available on the SOLACTIVE website:
https://www.solactive.com/documents/correction-policy/.

4.5. Market Disruption
In periods of market stress the INDEX ADMINISTRATOR shall calculate the INDEX following predefined and
exhaustive arrangements as described in the SOLACTIVE Disruption Policy, which is incorporated by
reference and available on the SOLACTIVE website:
https://www.solactive.com/documents/disruption-policy/. Such market stress can arise due to a
variety of reasons, but generally results in inaccurate or delayed prices for one or more INDEX
COMPONENTS. The determination of the INDEX may be limited or impaired at times of illiquid or
fragmented markets and market stress.

5. Miscellaneous..........................................................................................................................................................
5.1. Discretion
Any discretion which may need to be exercised in relation to the determination of the INDEX (for
example the determination of the Index Universe (if applicable), the selection of the INDEX
COMPONENTS (if applicable) or any other relevant decisions in relation to the INDEX) shall be made in
accordance with strict rules regarding the exercise of discretion or expert judgement by the INDEX
ADMINISTRATOR.

5.2. Methodology Review
The methodology of the INDEX is subject to regular review, at least annually. If a change of the
methodology has been identified within such review (e.g. if the underlying market or economic
reality has changed since the launch of the INDEX or if the present methodology is based on obsolete
assumptions and factors and no longer reflects the reality as accurately, reliably and appropriately

31
Version 1.1 – 30 September 2025
as before), such change will be made in accordance with the SOLACTIVE Methodology Policy, which
is incorporated by reference and available on the SOLACTIVE website:
https://www.solactive.com/documents/methodology-policy/.

Such change in the methodology will be announced on the SOLACTIVE website under the Section
“Announcements”, which is available at https://www.solactive.com/news/announcements/. The
date of the last amendment of this INDEX is contained in this GUIDELINE.

5.3. Changes in Calculation Method
The application by the INDEX ADMINISTRATOR of the method described in this document is final and
binding. The INDEX ADMINISTRATOR shall apply the method described above for the composition and
calculation of the INDEX. However, it cannot be excluded that the market environment, supervisory,
legal and financial or tax reasons may require changes to be made to this method. The INDEX
ADMINISTRATOR may also make changes to the terms and conditions of the INDEX and the method
applied to calculate the INDEX that it deems to be necessary and desirable in order to prevent obvious
or demonstrable error or to remedy, correct or supplement incorrect terms and conditions. The
INDEX ADMINISTRATOR is not obliged to provide information on any such modifications or changes.
Despite the modifications and changes, the INDEX ADMINISTRATOR will take the appropriate steps to
ensure a calculation method is applied that is consistent with the method described above.

5.4. Termination
The INDEX ADMINISTRATOR makes the greatest possible efforts to ensure the resilience and continued
integrity of its indices over time. Where necessary, the INDEX ADMINISTRATOR shall follow a clearly
defined and transparent procedure to adapt INDEX methodologies to account for changing
underlying markets (see Section 5.2 “Methodology Review”) in order to maintain continued
reliability and comparability of the indices. Nevertheless, if no other options are available the
orderly cessation of the INDEX may be indicated. This is usually the case when the underlying market
or economic reality, which an index is set to measure or to reflect, changes substantially and in a
way not foreseeable at the time of inception of the INDEX, the index rules, and particularly the
selection criteria, can no longer be applied coherently or the INDEX is no longer used as the
underlying value for financial instruments, investment funds and financial contracts.

The INDEX ADMINISTRATOR has established and maintains clear guidelines on how to identify situations
in which the cessation of an index is unavoidable, how stakeholders are to be informed and
consulted and the procedures to be followed for a termination or the transition to an alternative
index. Details are specified in the SOLACTIVE Termination Policy, which is incorporated by reference
and available on the SOLACTIVE website: https://www.solactive.com/documents/termination-
policy/.

5.5. Index Committee
An index committee composed of staff from the INDEX ADMINISTRATOR and its subsidiaries (the “ INDEX
COMMITTEE ”) is responsible for decisions regarding any amendments to the rules of the INDEX. Any

32
Version 1.1 – 30 September 2025
such amendment, which may result in an amendment of the GUIDELINE, must be submitted to the
INDEX COMMITTEE for prior approval and will be made in compliance with the Methodology Policy,
which is available on the SOLACTIVE website: https://www.solactive.com/documents/methodology-
policy/.

33
Version 1.1 – 30 September 2025
6. Definitions
“ ASK PRICE ” in relation to a CALCULATION DAY t and OPTION 𝑂, shall mean (i) the LISTED ASK PRICE , if the
OPTION 𝑂 is an ELIGIBLE LISTED OPTION calculated in accordance with Section 2.2.2; or (ii) otherwise, the
price estimated in accordance with Section 4.2.2

“ BENCHMARK REGULATION ” shall have the meaning as defined in Section “Introduction”.

“ BID PRICE ” in relation to a CALCULATION DAY t and OPTION 𝑂, shall mean (i) the LISTED BID PRICE, if the
OPTION 𝑂 is an ELIGIBLE LISTED OPTION calculated in accordance with Section 2.2.2; or (ii) otherwise, the
price estimated in accordance with Section 4.2.2.

“ BMR ” shall have the meaning as defined in Section “Introduction”.

“CALCULATION DAY” means a weekday on which each of NYSE and CBOE are open for business.

“CASH AMOUNT” shall have the meaning as defined in Section 4.1.3.

“CUMULATIVE DISTRIBUTION FUNCTION” defines the standard normal distribution.

“CONTINUING OPTION PORTFOLIO” has the meaning given to it in Section 4.1.2.

“DAY COUNT FRACTION” has the meaning given to it in Section 4.2.5

“DELTA” shall have the meaning given to it in Section 4.2.4

“DISCOUNT FACTOR” has the meaning given to it in Section 4.2.6

“ELIGIBLE LISTED EXPIRATION DATE” shall have the meaning given to it in Section 2.2.1

“ELIGIBLE LISTED OPTION” has the meaning given to it in Section 2.2.1

“ELIGIBLE LISTED STRIKE” has the meaning given to it in Section 2.2.1

“EXCHANGE” means any of the New York Stock Exchange (“ NYSE ”) or the Chicago Board Options
Exchange (“ CBOE ”).

“EXCHANGE ASK PRICE” of an OPTION or HEDGE INSTRUMENT means the ask price sourced from the relevant
exchange.

“EXCHANGE BID PRICE” of an OPTION or HEDGE INSTRUMENT means the bid price sourced from the relevant
exchange.

“EXPIRATION DATE” is defined in relation to an OPTION, FUTURE CONTRACT or FORWARD and is the date on
which such instrument expires.

“EXPONENTIAL FUNCTION” means the exponential function to the basis of Euler’s Number e.

“FORWARD” has the meaning given to it in Section 4.2.8

“FRICTION” is defined in relation to an OPTION and has the meaning given to it in Section2.1.

“FUTURE CONTRACT” means a listed futures contract in respect of the UNDERLYING ASSET.

“ GUIDELINE ” shall have the meaning as defined in Section “Introduction”.

“HALF TRADING DAY” means a CALCULATION DAY on which an early market close is announced by the
relevant Exchange.

“HEDGE INSTRUMENT” has the meaning given to it in Section 2.3

34
Version 1.1 – 30 September 2025
“ INDEX ” shall have the meaning as defined in Section “Introduction”.

“ INDEX ADMINISTRATOR ” shall have the meaning as defined in Section “Introduction”.

“ INDEX COMMITTEE ” shall have the meaning as defined in Section 5.5

“ INDEX COMPONENTS ” means, with respect to the INDEX and a Calculation Day, all the OPTIONS in the
CONTINUING OPTION PORTFOLIO on such day.

“ INDEX OWNER ” shall have the meaning as defined in Section “Introduction”.

“IMPLIED VOLATILITY” has the meaning given to it in Section 4.2.10

“ LISTED OPTION ” means an OPTION that is listed on an EXCHANGE.

“ LIVE DATE ” means 13 th March 2024.

“MAXIMUM FUNCTION” means, when followed by a series of amounts inside brackets, whichever is the
larger of the amounts separated by a comma inside those brackets.

“MID PRICE” in relation to a CALCULATION DAY t and OPTION 𝑂, shall mean (i) the LISTED MID PRICE, if the
OPTION �� is an ELIGIBLE LISTED OPTION calculated in accordance with Section 2.2.2; or (ii) otherwise, the
price estimated in accordance with Section 4.2.2

“NATURAL LOGARITHM FUNCTION” is the inverse of the EXPONENTIAL FUNCTION.

“NUMBER OF UNITS” is defined in relation to an OPTION and is the quantity or number of OPTIONS.

“OPTION” means a derivative that securitizes the right but not the obligation to buy (being OPTION
TYPE Call or a “ CALL OPTION ”) or sell (being OPTION TYPE Put or a “ PUT OPTION ”) a pre-defined reference
instrument relating to a position in respect of the UNDERLYING ASSET, on a pre-defined day (being
EXPIRATION DATE 𝑇𝐸), for a pre-defined price (being STRIKE PRICE 𝐾).

“ OPTION TYPE ” shall mean the type of OPTION 𝑂, which can be either “Call” or “Put”.

“PAYOUT” has the meaning given to it in Section 4.2.1.

“PORTFOLIO MARK-TO-MARKET” has the meaning given to it in Section 4.1.1.

“PREMIUM” has the meaning given to it in Section 4.2.2.

“PREMIUM PAID” has the meaning given to it in Section 4.1.4.

“ REFINITIV ” is a data provider being a subsidiary of London Stock Exchange.

“ ROLLING FUTURE DATE ” is a Calculation Day that is five Calculation Days prior to the EXPIRATION DATE of
the closest FUTURE CONTRACT to expire.

“ SOLACTIVE ” shall have the meaning as defined in Section “Introduction”.

“START DATE” means 3 rd January 2017.

“STRIKE PRICE” is defined in relation to an OPTION and is the strike price specified in respect of such
OPTION.

“TRADE DATE” means, in relation to an OPTION 𝑂, the CALCULATION DAY t on which the position in respect
of such OPTION is notionally traded.

“TRANSITION DATE” means 12 th May 2022.

“ UNDERLYING ASSET” or “SPX INDEX” ” means the S&P 500 Index.

35
Version 1.1 – 30 September 2025
“UNDERLYING INDEX CLOSING LEVEL” in relation to a CALCULATION DAY t means the official close of the
UNDERLYING ASSET on that day, identified by its RIC .SPX.

“UNWIND DATE” is defined in relation to an OPTION and is the date on which such OPTION unwinds.

“UNWIND VALUES” has the meaning given to it in Section 4.1.6.

“USD” means United States Dollars.

“ VARIANCE STRIKE “ has the meaning given to it in Section 2.1.1

“VEGA” has the meaning given to it in Section 4.2.4

36
Version 1.1 – 30 September 2025
7. Versioning
VERSION DATE DESCRIPTION
1.0
December 17th,
2024
Initial Guideline creation ( initial version )
1.1 September 30
th,
2025
Add Indicative Index.
Table 6 Versioning

Contact..............................................................................................................................................................................
Solactive AG
German Index Engineering
Platz der Einheit 1
60327 Frankfurt am Main
Germany

Tel.: +49 (0) 69 719 160 00
Fax: +49 (0) 69 719 160 25
Email: info@solactive.com
Website: http://www.solactive.com

© Solactive AG
