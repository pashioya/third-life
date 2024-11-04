# Third Life
Trying to create a stable simulation of worlds based on real research.

Bit by bit we are adding many different aspects of reality with the goal of creating a stable environment where humans can live and prosper. Anthing we add is supported by some kind of research which can be found in the [Gitlab Wiki](https://gitlab.com/groups/kdg-ti/the-lab/teams-23-24/third-life/-/wikis/Project-Overview).




### Project Setup
The `core` project contains the main bevy project with the simulation. This is the one that needs to be compiled to create an executable for the simulation. But for the compilation to work, the `core-macros` project needs to be downloaded and put in the same directory as the `core` project. This is because `core-macros` is a dependency to the `core` project. The `core-macros` project has to be separate because of how `rust` macros are compiled.

Side note: the `core-macros` project could be imported directly from Gitlab but that would require authentication which in my opinion is more annoying than just downloading the project.



# Research
## Wealth Infrastructure and Evironment

Infrastructure is a very general concept that can also go very much into detail but in general I would define it as a supporting concept, so for example, People can go to the toilet without a toilet but it sure as hell is easier with a toilet. In that notion infrastructure is in general things that improve other things.

That said implementing it seems more complex then expected. There are so many different sides to infrastructure that I find its almost useless to try and make sense of it. But since we still need to make sense of this mess we made the decision to tie infrastructure in large part to "GDP". That means that the amount of money a Colony has determines how good their infrastructure is. But now the question becomes, how does a colony make money. The relationship is the following:

![image](uploads/de6f79ca7c0eea22f99fd7b1e05ba66c/image.png)

The important thing to note here is that the flat rate that everyone is payed is constant or at least semi constant, meaning that what really matters is the proportion of people that have to work in food production compared to the people that don't have to. 

The exact ways in which this can be affected is still not completely clear but one of them could be how efficient the cooking is.

To come back to infrastructure, the way it will then work concretely is the following. Once we know how much of our wealth goes to paying salaries we can then create policies on how to split up the rest. Here is where we compare ourselves to the real world and how real governments decide to spend their money. By doing this comparison we can correlate real outcomes in different countries to their spending. This way we can for example determine where infant mortality should be related to % spending in sanitation infrastructure.

This means that the procedure will be: (1) finding a value that we would like to know (2) finding what that value is related to, things like spending but also things like food availability, whatever is needed. (3) And then finally defining the concrete behavior of the interaction through things like mathematical correlations but also decision trees.


## Here are some of the researched topics

### Heath care / Sanitation infrastructure

#### Sources

- [Healthcare in the US compared to other high income countries](https://www.commonwealthfund.org/publications/fund-reports/2021/aug/mirror-mirror-2021-reflecting-poorly)
- [Healthcare systems getting more value for money](https://www.oecd.org/economy/growth/46508904.pdf)
- [Dataset - Health expenditure as percent of GDP](https://www.statista.com/statistics/268826/health-expenditure-as-gdp-percentage-in-oecd-countries/)
- [Dataset - Health and health systems ranking by health index score](https://www.statista.com/statistics/1290168/health-index-of-countries-worldwide-by-health-index-score/)
- [Dataset - Mortality rate per 1.000 live births](https://data.worldbank.org/indicator/SP.DYN.IMRT.IN?locations=IT&view=map)

#### Conclusion

To get a stupidly simple starting point I decided to correlate both the `health index score` and the `infant mortality rate` to the respective countries mortality rate which gave me a result (that is probably not correct) more then good enough for my needs.

![image](uploads/820bc2697931b9f18371bdc6829b893e/image.png)

The countries are the following: 
```python
{'Denmark', 'Luxembourg', 'Germany', 'United Kingdom', 'Costa Rica', 'Ireland', 'New Zealand', 'Slovenia', 'Italy', 'Colombia', 'Spain', 'Australia', 'Sweden', 'Norway', 'Portugal', 'Latvia', 'Canada', 'Poland', 'Japan', 'Hungary', 'Finland', 'Iceland', 'France', 'Chile', 'Israel', 'Mexico', 'Belgium', 'Lithuania', 'Austria', 'Greece', 'United States', 'Netherlands', 'Estonia', 'Switzerland'}
```

and the full correlations are: 

Health index: 

```math
5.001857986634171 * \log_{10}{spending} + b: 68.60777916821428
```

Infant mortality rate:

```math
-0.00259713855544207 * \log_{10}{spending} + 0.009227181881168221
```


### General government spending

- [EU government expenditure by function](https://ec.europa.eu/eurostat/statistics-explained/index.php?)


### Wealth distribution 

#### Sources

- [Simulating the utopia of COMMONISM](https://link.springer.com/article/10.1007/s43253-023-00110-0)


# Environment

As shown above, there are some things that are somewhat easy to make imprecise predictions of, but the environment is definitely not one of them. As also mentioned above I will limit myself to single values to simplify the problem and tackle it piece by piece.

## Here are some of the researched topics

### Cow meat quality

#### Sources

- [European Union beef sector](https://www.europarl.europa.eu/RegData/etudes/BRIE/2022/733676/EPRS_BRI(2022)733676_EN.pdf)
- [EFSA: Better housing needed for dairy cows](https://www.efsa.europa.eu/en/news/efsa-better-housing-needed-dairy-cows-ducks-geese-and-quail-improve-welfare)
- [Minimum space requirements for cattle](https://bvajournals.onlinelibrary.wiley.com/doi/full/10.1002/vetr.2780)

#### Conclusion

The idea here is to tie the space available to cows to the quality of their meat. This stems from real research that points to higher stress and lower life qualities when cows are in cramped environments. To create some kind of correlation for this I need some kind of data which is hard to find...


### General Ecological footprint

- [Ecological footprint of European countries](https://www.eea.europa.eu/en/analysis/indicators/ecological-footprint-of-european-countries?activeAccordion=546a7c35-9188-4d23-94ee-005d97c26f2b#footnote-IGIDNTMX)


# Trading

There are a number of things that need to be modeled with functions in this space. I will add the gradually as I need them.

## Selling and Buying prices

When a planet has resources, it needs these for its own population but might also want to sell them if there is a surplus. The question now is, what will the price of these resources be and how will it change depending on the fluctuation of demand/supply.

Doing a quick google search I found a few different things:

- [this](https://www.ecb.europa.eu/pub/pdf/scpwps/ecbwp1184.pdf) (very technical) whitepaper that goes into waaay to much detail regarding optimal prices compared to actual pricing
- [this medium article](https://medium.com/teconomics-blog/how-to-get-the-price-right-9fda84a33fe5) which helps to find the optimal price
- then also [this one pager](https://www.math.ksu.edu/\~rekha/Elasticity.pdf) to get more information about details

Some thoughts I had while looking at these are:

* There has to be a point in which the planet decides to switch from selling to buying, this can be determined by each planet (but has a def. val) and is probably defined by population (meaning how much the population needs of that)
* After deciding whether to buy or to sell there are two functions that define the price in the two situations how much surplus (or the contrary) is there

All of this said I have to say that I struggle to make these realistic formulas work in such a limited environment so I'm simplifying them to fit my world

##### Selling Surplus

The more surplus `s` (s \> 0) there is the cheaper the price should get since we want to get rid of it and no one is buying it from us. Here I tried to find something that will enter at a determined price (price when there is 1 surplus) `mod2` (mod2 \> 0) and then slowly approach cheaper and cheaper prices for the surplus. How quickly the cheaper prices are approached is determined by `mod1` (mod1 \> 0)

> price at which to sell = e^( -s/mod1 + mod2)

##### Buying for missing goods

The more we are off from the ideal situation defined by a arbitrary value the higher the willingness to buy, so the prices should skyrocket.

So This is what I cam up with. `m_g` (m_g \> 0) is the number of missing goods. `mod` (mod \> 0) is a modifier which determines how quickly the price rises depending on missing goods. This function passes through 0 and then follows a logarithmic path meaning it never hits a limit.

> price willing to pay = ln(m_g + e) \* mod - mod

# Rising and Falling Population

## Sources

#### [Population growth and the food crisis](https://www.fao.org/3/U3550t/u3550t02.htm)

This Source mentions four different components:

> First, life-styles, incomes and social organization determine levels of consumption.

I would translate this as general wealth of a population but could also be more fine grained where single individuals in a population have differing amounts of wealth leading to differences in reactions.

> Second, the technologies in use determine both the extent to which human activities damage or sustain the environment and the amount of waste associated with a given level of consumption. Poverty may prevent the adoption of more appropriate technologies that could halt or slow down environmental degradation. These two factors determine the impact on individuals.

This point is harder to implement at first as it assumes that something like technological differences are a thing and that there are differences within one population. If we get to the point where that is a thing this would be possible.

> Inequality enters as a third factor when, for example, most land is in large holdings and the poor are forced to live on smallholdings or in marginal areas.

Although different in the nuances, I think this is related to the first point of differing wealth but expressed as differing resources.

> A fourth factor, population, acts as the multiplier that determines the total impact. Population is always part of the equation. For any given type of technology, level of consumption or waste and poverty or inequality, the more people there are, the greater the impact on the environment is and, in turn, the greater the impact on food production capacity will be.

As expected the relation of population to food production is very important. A population that is very dependent on production that can barely sustain it will have a harder time then a population that has a lot of surplus

#### [Human Carrying Capacity Is Determined by Food Availability](https://www.researchgate.net/publication/227269417_Human_Carrying_Capacity_Is_Determined_by_Food_Availability)

This paper does not concern itself with the reduction of population directly but more with the relationship of the lands carrying capacity relative to population size, for this the following formula is stated:

```math
P(t+\Delta t)=\frac{K(t)}{1+(\frac{K(t)}{P(t)}-1)e^{-r\Delta}}
```

Where $`P`$ is the population size $`K`$ the lands carrying capacity and $r \> 0$ the reproduction rate.

#### [Social determinants of human reproduction](https://academic.oup.com/humrep/article/16/7/1518/693439)

Reading trough the paper I come to the conclusion that there are a number of factors influencing the fertility rate. First of all we need to understand that there are different 'fertility regimes'.

> In discussing the economics of human fertility behaviour, it is necessary to differentiate between three different stages in any society's economic development, centred on what is known as the \`demographic transition'.

With that we mean that agrarian societies almost have a 'demand' for children but as urbanization increases this demand decreases. Then again, technologically advanced societies also have higher mortality rates which counteract the higher reproduction rates. Once urbanization and technology levels approach a maximum economic factors become the deciding factor.

There are a number of hypothesis on how exactly economic factors influence reproduction. Some are:

* The perceived time cost of a child meaning that if both parents work the perceived cost if higher compared to if one of the parents stays at home.
* Families have children when they feel their income surpasses some threshold. This is arbitrarily determined by the Family and their experience growing up.

#### [How long can someone live without food or water](https://www.virtualhospice.ca/en_US/Main+Site+Navigation/Home/Support/Support/Asked+and+Answered/Nutrition/How+long+can+someone+live+without+food+or+water\_.aspx)

> Our bodies tend to have several weeks worth of reserve energy from food stores,

> When someone is no longer taking in any fluid ... then this person may live as little as a few days or as long as a couple of weeks.

I think these two statements are good enough.

#### [Life expectancy and the environment](https://www.sciencedirect.com/science/article/pii/S0165188909002164?casa_token=zfHR0sAGyhwAAAAA:tJJbWGNpDmUQ8PtKDPGfi9mde2p4sCnVl6kL-NKTGzoDOYYm9BDYCk5Zi3POBO9xlRdyTxOW#fig1)

This paper very much goes into detail on how the environment of a person influences their life expectency, so much actually that their opening argument is enough for us to use. Here they start off by referring to the `EPI` and how it correlates to life expectancy, not a lot but enough.

> This synthetic indicator takes into account both “environmental health”, as defined by child mortality, indoor air pollution, drinking water, adequate sanitation and urban particulates, and “ecosystem vitality”, which includes factors like air quality, water and productive natural resources, biodiversity and sustainable energy

#### [The Linear Link: Deriving Age-Specific Death Rates from Life Expectancy](https://www.mdpi.com/2227-9091/8/4/109)

This paper very nicely shows how the probability of death increases depending on the life expectancy, and how that probability acts towards the extremes.

## Some Ideas

While doing this research I concluded a few different things. (1) First would be the matter of how much does a population reproduce/grow with the available resources and then (2) second, if there is a food shortage how does the population collapse, do people die evenly or is there a pattern. (3) That said there are some things that fall away from my research. I'm not here to determine how many people a planet can support.

## Conclusion

As I already mentioned there are two parts two this so the conclusion will also be in tow parts:

### Reproduction rate:

The paper '[Social determinants of human reproduction](https://academic.oup.com/humrep/article/16/7/1518/693439)' goes quite into depth about all the aspects of reproduction rates but for now I will only point the most basic aspects. Reproduction is indirectly a factor of `urbanization` with less urbanized societies having less of a `'demand' for children` but at the same time the more urbanized the more `economic factors` play a role in the reproduction rate. Specifically the difference between the parents and their children. Meaning that if your parents are wealthier then you are you are less likely to deem yourself stable enough to have children and lastly `technological advancement` plays a role in the survival rates of the newborns.

To put this in concrete terms I created a function which could give us the reproduction rate. This is just a first idea but I think it is a good start. First lets define some variables:

* $`D`$ is the demand, which I would say is anywhere between $`2.1`$ and maybe $`7`$ in extreme cases.
* $`E`$ for the economy, this could just be the whole planets economy but also a single families economy, it doesn't really matter on which scale this number is
* $`s`$ for the survivability, which is tightly linked to technological level of the planet 0\<s\<1
* $`u`$ the level of urbanization of the planet 0\<u\<1

```math
economic=2.1u\frac{E_t}{E_{t-1}}
```

```math
demand=D(1-u)
```

```math
r=economic*demand*s
```

So to explain how the before mentioned points find themselves in this function: (1) The economic point is expressed as a default demand of 2.1 discounted by the difference in economy and the urbanization. (2) As for the demand it is simply discounted by the inverted urbanization to fall off towards more urbanization (3) Finally everything is multiplied by a survivability rate.

This is very much a fictional function but it is still somewhat based on reality.

### Chance of pregnancy and miscarriages
I found lists for both the changes of getting pregnant and having a miscarriage at specific ages from a number of sources cross validating and ensuring relatively accurate numbers, eventually arriving at the following: 

- [Eugin: Probability of Pregnancy By Age](https://www.eugin.co.uk/what-is-your-probability-of-pregnancy-by-age/)
- [Mira-Care: Chance of Pregnancy By Age](https://www.miracare.com/blog/your-chances-of-pregnancy-by-age/)
- [Advance-Fertility: Age - Female Fertility Relationship](https://advancedfertility.com/2020/09/11/age-and-female-infertility-fertility-tests-of-egg-supply/)
- [Evidence on: Pregnancy at Age 35 and Older](https://evidencebasedbirth.com/advanced-maternal-age/#:\~:text=People%20aged%2018%20to%2034,8.1%20per%201%2C000%2C%20or%200.81%25)

| Age | Pregnancy Rate \[%\] | Miscarriage Rate \[%\] |
|-----|----------------------|------------------------|
| 18 | 84 | 17 |
| 19 | 84 | 17 |
| 20 | 84 | 11 |
| 21 | 84 | 11 |
| 22 | 84 | 11 |
| 23 | 84 | 11 |
| 24 | 83 | 11 |
| 25 | 81 | 10 |
| 26 | 80 | 10 |
| 27 | 78 | 10 |
| 28 | 75 | 10 |
| 29 | 72 | 11 |
| 30 | 69 | 11 |
| 31 | 66 | 11 |
| 32 | 63 | 11 |
| 33 | 61 | 11 |
| 34 | 60 | 11 |
| 35 | 58 | 17 |
| 36 | 57 | 17 |
| 37 | 52 | 17 |
| 38 | 50 | 17 |
| 39 | 49 | 17 |
| 40 | 47 | 33 |
| 41 | 46 | 33 |
| 42 | 36 | 33 |
| 43 | 30 | 33 |
| 44 | 26 | 33 |
| 45 | 20 | 47 |

I then took the table and attempted to graph an equation that best fits the points. eventually reaching the following using a \[Quartic Regression Model\].

- Y represents the \[Percentage Chance of Pregnancy\]
- X represents the age of the individual.

```math
y = -0.0005893368566x^(4) + 0.0730945581099x^(3) - 3.3813849411076x^(2) + 66.904528373158x - 390.6749280259455
```

The model has a R squared of 0.99 and the points are a near perfect fit.

![image](uploads/9bc7186f8724b8cbf395544378cccf8d/image.png)


![image](uploads/386bf92a2e691cef9e7b2ef4725a382f/image.png)

### Death rate

In the case that there are missing resources the following rules will apply:

* Water: 5 days without water before death
* Food: 3 weeks without food before death

### Life expectancy

In this case we can just directly map life expectancy with the EPI which is made up of

* environmental health
  * child mortality
  * indoor air pollution
  * drinking water
  * adequate sanitation
  * urban particulates
* ecosystem vitality
  * air quality
  * water resources
  * productive natural resources
  * biodiversity
  * sustainable energy

All of these are $`0`$to $`1`$ values and map to a $`0`$ to $`100`$ age scale.

Finally the actual probability of death will be approximated with a [sigmoid function](https://en.wikipedia.org/wiki/Sigmoid_function) like:

```math
\frac{1}{1+e^{\frac{-age+life\_exp}{spread}}}
```

where the spread is how far to the left and right the probability distribution goes.


# Food Production

# Food production

https://www.mdpi.com/2079-9276/5/4/47

Interesting study that supplies us with a nice table for different types of food production.

![food-production-table.png](uploads/8eabe1994a6e6d3370bd8ea96762535e/food-production-table.png)

In a modern Mechanized system seems the more limiting factor is area left to agriculture. the table above gives us the area needed for vegetable products.

## beef production

https://fefac.eu/newsroom/news/the-actual-size-of-european-livestock-farms/

- average holding is 34 hectares
- average size of herd 47 animals

For animal products the minimum size per animal is based on limits from a government.

from the european reccomendation for dairy cows. https://www.efsa.europa.eu/en/plain-language-summary/welfare-dairy-cows

* For indoor housing, a total indoor area – including lying area – of at least 9 m2/cow should be provided.

https://u.osu.edu/beef/1999/03/10/mating-capacity-of-bulls-bull-to-cow-ratio/ bull to cow ration = 1:20-1:30 will use 1:25

https://www.beefresearch.ca/topics/calving-seasons/ calving season can be chosen. will use spring calving to coincide with wheat harvest

https://www.steakboyz.com/howmanysteaks/how-many-steaks-from-a-cow total guesstimate kg of meat from a single cow = 250kg

time to harvest a cow 6.25 hours at full mechanization

## pork production

from the european law for pigs. https://eur-lex.europa.eu/EN/legal-content/summary/protection-of-pigs.html

* between 0.15 m<sup>2</sup> for pigs weighing less than 10 kg and 1 m<sup>2</sup> per animal over 110 kg;
* 1.64 m<sup>2</sup> per gilt;
* 2.25 m<sup>2</sup> per sow;
* 6 m<sup>2</sup> for a boar (10 m<sup>2</sup> if the boar is used for natural services).

https://australianpork.com.au/about-pig-farming/stages-pork-production

- Pig Farrow season is July-Sept
- 5-6 months to reach slaughter age

Average Farm Holding in europe 17.4 ha in 2020

https://ec.europa.eu/eurostat/statistics-explained/SEPDF/cache/73319.pdf

## Useful conversions

1 ha = 10,000 m<sup>2</sup>



# Food Consumption

# Citizen Nutritional Needs

In order to ensure the citizen nutritional requirements are met we decided to research actual nutritional needs of humans and would like to model it as accurate to the real world as possible.

## Water

https://www.mayoclinic.org/healthy-lifestyle/nutrition-and-healthy-eating/in-depth/water/art-20044256

* About 15.5 cups (3.7 liters) of fluids a day for men
* About 11.5 cups (2.7 liters) of fluids a day for women

## Possible Basic Initial Implementation

After some research I found a couple mathematical formulas that nutritionist use to approximate a individuals needs. The formula calculates the `Basic Metabolic Rate` or `Basic Energy Expenditure`.

`BMR` or `BEE` is essentially a formula that calculates the amount of energy per unit time that a human needs to keep their body functioning. it is measured in joule/second or watts.

Details For the formula can be found on the paper [Comparison of the Harris-Benedict Equation](https://www.ncbi.nlm.nih.gov/pmc/articles/PMC7784146/pdf/JAFES-33-2-152.pdf)

`BEE = 655 + (9.6 x weight in kg) + (1.8 x height in cm) - (4.7 x age in years)`

The formula calculates the Basal Metabolic Rate / Basal Energy Expenditure of an individual by taking into account their weight height and age. The formula can also be tweaked and scaled depending on the Activity factors of the person. as is shown in the following document: [Common Nutrition Formulas and Calculations](https://www.anfponline.org/docs/default-source/supplemental-textbook-material/nf2015/nutrition-common-formulas-and-calculations-(2).pdf):

Our Population would at least in the beginning be very active at all times. so the BEE would have to be multiplied by 0.5 making out final formula:

`BEE = 655 + (9.6 × weight in kg) + (1.8 × height in cm) - (4.7 × age in years) ÷ 2`

## Future Complex implementation

The following paper: [A Mathematical Model of Food Intake](https://www.aimspress.com/article/id/600171a2ba35de6310fafa52), describes a mathematical model for food intake. It takes into account a number of parameters which could potentially be modelled although with heavy abstractions since the paper is a little too in-depth and contains parameters that aren't going to be modeled within our simulation such as `Appetite` or `Ghrelin Plasma Concentration`

The Actual model is a dynamic system of complex differential and algebraic equations that alternate between a feeding state and a fasting state. The red arrows are positive effects while the blue arrows are negative effects. Each Symbol is its own equation, all equations can be found in the [paper](https://www.aimspress.com/article/id/600171a2ba35de6310fafa52).

All in all the implementation of this fully fledged formula is absolutely doable but would require a lot of time due to its complexity as well as the need to abstract away the parts we cannot model in our simulation. ![image](uploads/9fa922f1bf9360dcc064d1ffb21a3b03/image.png)

## Final equations

To determine the daily caloric intake of a person we can use the equations found [here](https://en.wikipedia.org/wiki/Basal_metabolic_rate), which are:
- for men: 66.4730 + 13.7516 * kg + 5.0033 * cm – 6.7550 * years
- for women: 655,0955 + 9,5634 * kg + 1,8496 * cm – 4,6756 * years

## Food conversion

After determitating the amount of calories that need to be consumed by the citizens there is a second conversion that needs to be made. The human stomach cannot make use of every single Joule of energy in the piece of Meat, meaning that there is a conversion rate between a 1kg of beef and the amount of calories found in it.

The ranges in the calories are there because different factors influence the amount of calories that can be extracted and this is a thing we can play on in the simulation.

| food | starting weight | resulting calories |
|------|-----------------|--------------------|
| beef | 1kg | 2000-3000 cal |
| weat | 1kg | 3000-3500 cal |

### more detailed nutriends

#### [Beef](https://www.vinmec.com/en/news/health-news/nutrition/how-much-protein-is-in-beef/)
- 260 grams of protein
- 120 grams of fat

#### [Weath](https://www.webmd.com/diet/health-benefits-of-wheat)
- 150 grams of protein
- 106 grams of dietary fiber
- 712 grams of carbohydrates

As for the ratio of nutriens that need to be consumed by a human, according to [this](https://www.acefitness.org/resources/pros/expert-articles/5904/how-to-determine-the-best-macronutrient-ratio-for-your-goals/) website the following ratio is true for a human doing medium to high intensity training during the day:
- 45 to 55 percent total carbohydrates
- 20 to 30 percent total protein 
- 30 percent total fat

The default ratio for our simulation will be: 
- 50% carbs
- 20% protein
- 30% fat

# Global Hunger Index 

### What is the Global Hunger Index

Global hunger index (**GHI**) is a tool designed to comprehensively measure and track hunger on a global, national, or even just a regional level., reflecting multiple dimensions of hunger over time.

This makes it perfect for measuring just how much hunger there is at any point in time. and use it to adjust the appropriate states in the simulation. 

### How the GHI Is Calculated

Each country’s GHI score is calculated based on a formula that combines four indicators that together capture the multidimensional nature of hunger:

*Undernourishment:*  the share of the population whose caloric intake is insufficient;

*Child Stunting*: the share of children under the age of five who have low height for their age, reflecting chronic undernutrition;

*Child Wasting: * the share of children under the age of five who have low weight for their height, reflecting acute undernutrition; 

*Child Mortality: * the share of children who die before their fifth birthday, reflecting in part the fatal mix of inadequate nutrition and unhealthy environments.

$\frac{Prevalence of undernourishment}{80} * x 100 = {standardized  Undernourishment  Value}$

$\frac{Child stunting rate}{70} * x 100 = {Standardized Child Stunting Value}$

$\frac{Child wasting rate}{30} * x 100 = {Standardized Child Wasting Value}$

$\frac{Child mortality rate}{30} * x 100 = {Standardized Child mortality Value}$


the following describes the value of the GHI score
![image](uploads/378eeaaa47b94cc965fab146d8c113ac/image.png)


### Factors Affected By a High GHI Score

- Pregnancy chances are lowered 
- Height growth on a citizen to citizen basis are lowered 
- Weight Changes on a citizen to citizen basis are lowered
- Children mortality rates are increased 
- ...many more systems are affected by association to the above changes.


