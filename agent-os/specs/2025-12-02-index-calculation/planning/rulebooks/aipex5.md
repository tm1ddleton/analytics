Version 1.

09 August 2019

AI Powered US Equity Index

TAB LE O F CO N TE N TS
Introduction
Index Specifications
1.1. Short Name and Isin.............................................................................................................................................................................................
1.2. Initial Value
1.3. Distribution
1.4. Prices and Calculation Frequency
1.5. Publication
1.6. Historical Data
1.7. Licensing..................................................................................................................................................................................................................
Calculation of the Index
2.1. Index Formula
2.2. Realized Volatility Calculation
2.3. Accuracy
Miscellaneous
3.1. Discretion
3.2. Methodology Review
3.3. Changes in calculation method
3.4. Termination
3.5. LIBOR Unavailability and Cessation
3.6. Oversight
Definitions
Contact
Introduction
This document (the “Guideline”) is to be used as a guideline with regard to the composition, calculation and
maintenance of the AI Powered US Equity Index (the “Index”). Any changes made to the Guideline are
subject to the approval of a committee, as specified in Section 3. 5. The Index is administered, calculated
and published by Solactive AG (“Solactive”) assuming the role as administrator (the “Index Administrator”)
under the Regulation (EU) 2016/1011 (the “Benchmark Regulation” or “BMR”). The name “Solactive” is
trademarked.

The Guideline and the documents referenced herein contain the underlying principles and rules regarding
the structure and operation of the Index. Solactive AG does not offer any explicit or tacit guarantee or
assurance, neither pertaining to the results from the use of the Index nor the level of the Index at any
certain point in time nor in any other respect. Solactive AG strives to the best of its ability to ensure the
correctness of the calculation. There is no obligation for Solactive AG – irrespective of possible obligations
to issuers – to advise third parties, including investors and/or financial intermediaries, of any errors in the
Index. The publication of the Index by Solactive AG does not constitute a recommendation for capital
investment and does not contain any assurance or opinion of Solactive AG regarding a possible investment
in a financial instrument based on this Index.

1. Index Specifications
The Index is owned by EquBot Inc. (“EquBot”) and administered, calculated, and published by Solactive
AG.

The Index is dynamically exposed to the AI Powered US Equity Base Index (TR) (the “Base Index”). This
exposure varies between 0% and 150% and may be adjusted on each Trading Day to aim to achieve a
volatility of 6% for the Index.

The Index is excess return, which reflects the weighted performance of the Base Index in excess of the
performance of the USD 3 - Month Libor rate.

The Index incorporates a fee of 0.50% per annum, deducted daily.

The Index is denominated in USD.

1.1. Short Name and Isin.............................................................................................................................................................................................
The Index is published under the following identifiers:

Name ISIN RIC Bloomberg Code
AI Powered US Equity Index DE000SLA8WA0 .AIPEX AIPEX Index
1.2. Initial Value
The Index is based on 1000 at the close of trading on 30 April 2004 (the “Start Date”).

1.3. Distribution
The Index is published on the website of the Index Administrator (www.solactive.com) and is, in addition,
available via the price marketing services of Boerse Stuttgart AG and may be distributed to all of its
affiliated vendors. Each vendor decides on an individual basis as to whether it will distribute or display the
Index via its information systems.

1.4. Prices and Calculation Frequency
The closing level of the Index is calculated on each Calculation Day based on the closing level of the Base
Index.

1.5. Publication
All specifications and information relevant for calculating the Index are made available on the Solactive
website: http://www.solactive.com.

1.6. Historical Data
Historical data will be maintained from the launch of the Index on 09 August 2019.

1.7. Licensing..................................................................................................................................................................................................................
Licenses to use the Index as the underlying value for derivative instruments are issued to stock exchanges,
banks, financial services providers and investment houses by EquBot.

2. Calculation of the Index
2.1. Index Formula
The closing level of the Index in respect of each Calculation Day after the Start Date (“Calculation
Day t”) shall be calculated in accordance with the following formula:

ILt=ILr×( 1 +Expr×(
BILt
BILr
− 1 −rr×
DCr,t
360
) −Fee×
DCr,t
360
)
Where:

ILt Means the closing level of the Index in respect of Calculation Day t;

ILr Means the closing level of the Index in respect of the Trading Day immediately preceding
Calculation Day t;

BILt Means the closing level of the Base Index in respect of Calculation Day t;

BILr Means the closing level of the Base Index in respect of the Trading Day immediately
preceding Calculation Day t;

rr Means the USD 3-Month Libor Rate in respect of the Trading Day immediately preceding
Calculation Day t;

DCr,t Means the number of Calendar Days from (but excluding) the Trading Day immediately
preceding Calculation Day t to (and including) Calculation Day t;

Fee Means 0.50% (per annum);

Expr Means the exposure in respect of the Trading Day immediately preceding Calculation
Day t, calculated in accordance with the following formula:

Expr=min(Max Exposure,
Target Volatility
RVr− 1
)
Where:

Max Exposure means 150%;
Target Volatility means 6%;
RVr− 1 means the Realized Volatility (as defined in section 2.2) in respect to of the
Calculation Day immediately preceding the Trading Day immediately preceding
Calculation Day t.
2.2. Realized Volatility Calculation
The realized volatility (the “Realized Volatility”) in respect of each Calculation Day (“Calculation Day
t”) shall be calculated in accordance with the following formula:

RVt=max
(
(^) √^252
21
∗∑(ln(
BILt−i
BILt−i− 1

))
2
20
i= 0
,√
252
63
∗∑(ln(
BILt−i
BILt−i− 1
))
2
62
i= 0
)
Where:

RVt Means the Realized Volatility in respect of Calculation Day t;

BILt−i Means the closing level of the Base Index in respect of the Calculation Day falling i
Calculation Days prior to Calculation Day t;

BILt−i− 1 Means the closing level of the Base Index in respect of the Calculation Day falling i+
Calculation Days prior to Calculation Day t.

2.3. Accuracy
The level of the Index will be rounded to 2 decimal places.

3. Miscellaneous
3.1. Discretion
Any discretion which may need to be exercised in relation to the determination of the Index) shall be made
in accordance with strict rules regarding the exercise of discretion or expert judgement.

3.2. Methodology Review
The methodology of the Index is subject to regular review, at least annually. In case a need of a change of
the methodology has been identified within such review (e.g. if the underlying market or economic reality
has changed since the launch of the Index, i.e. if the present methodology is based on obsolete
assumptions and factors and no longer reflects the reality as accurately, reliably and appropriately as
before), such change will be made in accordance with the Solactive Methodology Policy which is
incorporated by reference and available on the Solactive website:
https://www.solactive.com/documents/methodology-policy/.

Such change in the methodology will be announced on the Solactive website under the Section
“Announcement”, which is available at https://www.solactive.com/news/announcements/. The date of
the last amendment of this Index is contained in this Guideline.

3.3. Changes in calculation method
The application by the Index Administrator of the method described in this document is final and binding.
The Index Administrator shall apply the method described above for the composition and calculation of
the Index. However, it cannot be excluded that the market environment, supervisory, legal and financial or
tax reasons may require changes to be made to this method. The Index Administrator may also make
changes to the terms and conditions of the Index and the method applied to calculate the Index that it
deems to be necessary and desirable in order to prevent obvious or demonstrable error or to remedy,
correct or supplement incorrect terms and conditions. The Index Administrator is not obliged to provide
information on any such modifications or changes. Despite the modifications and changes, the Index
Administrator will take the appropriate steps to ensure a calculation method is applied that is consistent
with the method described above.

3.4. Termination
Solactive makes the greatest possible efforts to ensure the resilience and continued integrity of its indices
over time. Where necessary, Solactive follows a clearly defined and transparent procedure to adapt Index
methodologies to changing underlying markets (see Section 3 .2 “Methodology Review”) in order to maintain
continued reliability and comparability of the indices. Nevertheless, if no other options are available the

orderly cessation of the Index may be indicated. This is usually the case when the underlying market or
economic reality, which an index is set to measure or to reflect, changes substantially and in a way not
foreseeable at the time of inception of the index, the index rules, and particularly the selection criteria, can
no longer be applied coherently or the index is no longer used as the underlying value for financial
instruments, investment funds and financial contracts.

Solactive has established and maintains clear guidelines on how to identify situations in which the
cessation of an index is unavoidable, how stakeholders are to be informed and consulted and the
procedures to be followed for a termination or the transition to an alternative index. Details are specified
in the Solactive Termination Policy, which is incorporated by reference and available on the Solactive
website: https://www.solactive.com/documents/termination-policy/.

3.5. LIBOR Unavailability and Cessation
(a) In the event that the USD 3-Month LIBOR Rate does not appear on the applicable price source at
approximately 11:00 a.m., London time, on any Calculation Day, then Solactive will use for the calculation
of the Index the most recent available USD 3 Month LIBOR Rate published by the Thomson Reuters
Corporation.

(b) If Solactive determines that USD 3 Month LIBOR (1) is no longer representative as a measure of the
average rate at which banks are willing to borrow wholesale unsecured funds in the London interbank
market or (2) has been discontinued at any time, it will substitute for USD 3 Month LIBOR an industry-
accepted substitute or successor rate (the “LIBOR Successor Rate”), including any adjustment to or related
spread on such LIBOR Successor Rate, in each case in its sole discretion and in accordance with Section
3.6. In the event that Solactive determines, in its sole discretion, that there is no industry-accepted
substitute or successor rate and that there are no quotations provided as described in Section 3.5(a), then,
after consulting such sources as it deems reasonable, it will estimate the USD 3-Month LIBOR Rate in its
sole discretion from time to time to use as the LIBOR Successor Rate. Further, if Solactive subsequently
determines, in its sole discretion, that an industry-accepted substitute or successor rate has emerged or
otherwise become available, it will cease to estimate the LIBOR Successor Rate and instead substitute
such industry-accepted substitute or successor rate as provided in the first sentence of this Section 3.5(b).

If Solactive has determined a LIBOR Successor Rate (including any such adjustment and/or spread) in
accordance with the foregoing, Solactive in its sole discretion may also implement changes to the Index
rules as it determines are appropriate to account for such change to the LIBOR Successor Rate in a manner
that is consistent with industry-accepted practices for the LIBOR Successor Rate. Once Solactive chooses
a LIBOR Successor Rate, such LIBOR Successor Rate will be used in place of USD 3 Month LIBOR for all
calculations, and the term “USD 3-Month LIBOR Rate” as used in this methodology, shall be then deemed
to refer to the LIBOR Successor Rate.

3.6. Oversight
An oversight committee composed of staff from Solactive and its subsidiaries (the “Oversight Committee”)
is responsible for decisions regarding any amendments to the rules of the Index. Any such amendment,
which may result in an amendment of the Guideline, must be submitted to the Oversight Committee for
prior approval and will be made in compliance with the Methodology Policy, which is available on the
Solactive website: https://www.solactive.com/documents/methodology-policy/.

4. Definitions
A “Calculation Day” is any day on which the New York Stock Exchange (NYSE) is open for trading.

A “Trading Day” is any day on which the New York Stock Exchange (NYSE) and the NASDAQ are both open
for trading.

The “USD 3-Month Libor Rate” means, in respect of a Calculation Day, the ICE Libor USD 3 Month rate on
such Calculation Day, as published by the Thomson Reuters Corporation.

Contact
Solactive AG
German Index Engineering
Platz der Einheit, 1
60327 Frankfurt am Main
Germany

Tel.: +49 (0) 69 719 160 00
Fax: +49 (0) 69 719 160 25
Email: info@solactive.com
Website: http://www.solactive.com

© Solactive AG
