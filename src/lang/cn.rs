use crate::{num2words::Num2Err, Currency, Language};
use num_bigfloat::BigFloat;
use crate::lang::cn::ChineseRegion::Mainland;
use std::ops::{Index, IndexMut};

pub const fn arabic_num_to_char(digit:u8) ->Option<char>{
    if(digit<=9){
        Some((b'0'+digit) as char)
    }else if(digit<=35) {
        Some((b'a'+digit-10) as char)
    }else if(digit<=61) {
        Some((b'A'+digit-36) as char)
    }else{
        None
    }
}
pub const fn char_to_arabic_num(digit:char) ->Option<u8>{
    if(digit>='0'&&digit<='9'){
        Some((digit as u8)-b'0')
    }else if(digit>='a'&&digit<='z') {
        Some((digit as u8)-b'a'+10)
    }else if(digit>='A'&&digit<='Z') {
        Some((digit as u8)-b'A'+36)
    }else{
        None
    }
}
pub struct Digits{
    digits: Vec<u8>,
    max_digit: u8,
}
impl Digits{
    pub fn new(max_digit: u8) -> Digits{
        Digits{
            digits:Vec::new(),
            max_digit,
        }
    }
    #[inline]
    pub fn len(&self) -> usize{
        self.digits.len()
    }
    #[inline]
    pub fn append(&mut self, digit: u8){
        self.digits.push(digit);
    }
    pub fn from_string(digits_string: &str, max_digit: u8, convert_rule: fn(char) ->Option<u8>) -> Option<Digits>{
        let mut digit=Digits::new(max_digit);
        let mut digits_chars = digits_string.chars().collect::<Vec<char>>();
        digits_chars.reverse();
        for digit_char in digits_chars{
            digit.append(match convert_rule(digit_char) {
                Some(n) => n,
                None => return None,
            });
        }
        Some(digit)
    }
    pub fn to_string(&self,convert_rule: fn(u8)->Option<char>) -> Option<String>{
        let mut result = String::new();
        let digits_nums =self.digits.iter().rev();
        for digit in digits_nums {
            result.push(match convert_rule(*digit) {
                Some(c) => c,
                None => return None,
            } );
        }
        Some(result)
    }
    pub fn from_u64(mut value:u64, max_digit:u8) -> Digits{
        let mut digits=Digits::new(max_digit);
        while(value>0){
            digits.append((value % max_digit as u64) as u8);
            value/=max_digit as u64;
        }
        digits
    }
    pub fn from_big_float(mut num:BigFloat, max_digit:u8) -> Digits{
        if(num.is_zero()){
            return Digits::from_u64(0,max_digit);
        }
        let mut digits=Digits::new(max_digit);
        let num_sys = BigFloat::from(max_digit);
        while !num.is_zero() {
            digits.append((num % num_sys).to_u64().unwrap() as u8);
            num /= num_sys;
        }
        digits
    }
    #[inline]
    pub fn get_u8_array(&self)->&Vec<u8>{
        &self.digits
    }
}
impl Index<usize> for Digits{
    type Output = u8;

    fn index(&self, index: usize) -> &Self::Output {
        &self.digits[(index)]
    }
}
impl IndexMut<usize> for Digits{
    fn index_mut(&mut self, index: usize) -> &mut Self::Output{
        &mut self.digits[(index)]
    }
}
#[cfg(test)]
mod tests1 {
    use super::*;
    #[test]
    fn test_134(){
        let num="134";
        let digits = Digits::from_string(num,10,char_to_arabic_num).unwrap();
        assert_eq!(digits.to_string(arabic_num_to_char).unwrap(),num)
    }
    #[test]
    fn test_2978(){
        let num=2978;
        let digits = Digits::from_u64(num,10);
        assert_eq!(digits.to_string(arabic_num_to_char).unwrap(),num.to_string())
    }
    #[test]
    fn test_114514(){
        let num=114514;
        let digits=Digits::from_u64(num, 10);
        assert_eq!(digits.to_string(arabic_num_to_char).unwrap(),"114514");
    }
}

pub struct Chinese {
    //prefer 零 over 〇
    prefer_ling: bool,
    //prefer 一十 over 十 in the beginning
    prefer_yishi: bool,
    region: ChineseRegion,
}
pub enum ChineseRegion {
    Mainland,
    Taiwan,
    HongKong,
}

const DIGITS: [char; 9] = [
    '一','二','三','四','五','六','七','八','九',
];

const MEGAS: [&str; 12] = [
    "", "万", "亿", "兆", "京", "垓", "秭", "穰", "沟", "涧", "正", "载"
];
/// Traditional Chinese (繁體中文) megascale units
const MEGAS_TRAD: [&str; 12] = [
    "", "萬", "億", "兆", "京", "垓", "秭", "穰", "溝", "澗", "正", "載"
];

impl Chinese {
    pub fn new(prefer_ling: bool, prefer_yishi: bool, region: ChineseRegion) -> Self {
        Self {
            prefer_ling,
            prefer_yishi,
            region,
        }
    }

    fn currencies(&self, currency: Currency, plural_form: bool) -> String {
        //todo
        currency.default_string(plural_form)
    }

    fn megaunit(&self,place:usize)->Result<&'static str,Num2Err> {
        match self.region {
            Mainland => {
                if place < MEGAS.len() {
                    Ok(MEGAS[place])
                } else {
                    Err(Num2Err::CannotConvert)
                }
            }
            _ => {
                if place < MEGAS_TRAD.len() {
                    Ok(MEGAS_TRAD[place])
                } else {
                    Err(Num2Err::CannotConvert)
                }
            }
        }

    }
    fn digit_to_char(&self,digit:char)->char {
        if digit=='0' {
            self.zero()
        }else if digit>='1'&&digit<='9'{
            DIGITS[digit.to_digit(10).unwrap() as usize-1]
        }else{
            debug_assert!(false,"Char not 0 to 9: {}",digit);
            '?'
        }
    }
    fn zero(&self)->char{
        if self.prefer_ling {
            '零'
        }else{
            '〇'
        }
    }
    fn int_to_cardinal(&self,num: BigFloat) -> Result<String, Num2Err> {
        let mut text=String::new();
        if(num.is_zero()){
            return Ok(self.zero().to_string());
        }
        let num_chars=Digits::from_big_float(num, 10).to_string(arabic_num_to_char).unwrap()
            .chars().collect::<Vec<char>>();
        let last_digit= num_chars.len()-1;
        let mut zeros:usize=0;
        for (place,digit) in num_chars.iter().enumerate() {
            if *digit=='0' {
                zeros+=1;
            }else{
                zeros=0;
            }
            //if is first digit in the section or in the number, we should do some work
            if place%4==3||place==last_digit{
                //check if the unit is not used, like 1_0000_0000 do not display 万
                if zeros>=4 {
                    //if the whole section is 0, skip it
                    continue;
                }else{
                    //enter normal process
                    text.push_str(self.megaunit(place/4)?);
                    let mut section_zeros:usize=0;
                    for (section_place,section_digit) in num_chars
                        [if place==last_digit {last_digit-last_digit%4}else{place-3}..place+1].iter().enumerate(){
                        if *section_digit=='0' {
                            //check is there any hanging zeros, like 1001
                            //if it is the rightmost place of 0, we should consider adding 零
                            if section_zeros==0 {
                                //do not add 零 when it is the rightmost digit in the section
                                //but add it elsewhere
                                if section_place==0 {
                                    continue;
                                }else{
                                    text.push(self.zero());
                                }
                            }
                            //record one more zero
                            section_zeros+=1;
                        }else{
                            //enter normal number process
                            //add the approximate unit in section
                            match section_place{
                                0=>(),
                                1=>{
                                    text.push('十');
                                    //decide using 一十 or 十 in the beginning
                                    if !self.prefer_yishi&&place==last_digit&&*section_digit=='1' {
                                        continue;
                                    }
                                },
                                2=>{
                                    text.push('百');
                                },
                                3=>{
                                    text.push('千');
                                },
                                _=>{
                                    debug_assert!(false, "should not have section place of: {:?}",section_place );
                                }
                            }
                            text.push(self.digit_to_char(*section_digit));
                        }
                    }
                }
            }
        }
        Ok(text.chars().rev().collect::<String>())
    }

    fn float_to_cardinal(&self,num: BigFloat) -> Result<String, Num2Err> {
        let integer_part=num.int();
        let mut text=self.int_to_cardinal(integer_part)?;
        if num.frac().is_zero(){
            return Ok(text);
        }
        let decimal_part=Digits::from_big_float(num.frac(),10);
        //add decimal point
        match self.region{
            Mainland => {
                text.push('点');
            }
            _ => {
                text.push('點');
            }
        }
        //adding the decimal part
        for digit in decimal_part.to_string(arabic_num_to_char).unwrap().chars(){
            text.push(self.digit_to_char(digit));
        }
        Ok(text)
    }
}

impl Language for Chinese {
    fn to_cardinal(&self, mut num: BigFloat) -> Result<String, Num2Err> {
        if num.is_nan() {
            return Err(Num2Err::CannotConvert);
        }
        let mut text = String::new();
        if num.is_negative(){
            text.push('负');
            num = -num;
        }
        if num.is_inf(){
            text.push_str("无穷");
            return Ok(text);
        }
        text.push_str( self.float_to_cardinal(num)?.as_str());
        Ok(text)
    }

    fn to_ordinal(&self, mut num: BigFloat) -> Result<String, Num2Err> {
        todo!()
    }

    fn to_ordinal_num(&self, num: BigFloat) -> Result<String, Num2Err> {

        todo!()
    }

    fn to_year(&self, num: BigFloat) -> Result<String, Num2Err> {
        todo!()
    }

    fn to_currency(&self, num: BigFloat, currency: Currency) -> Result<String, Num2Err> {
        todo!()
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Lang, Num2Words, Currency};

    #[test]
    fn test_cardinal_basic() {
        assert_eq!(
            Num2Words::new(0)
                .lang(Lang::Chinese_Simp)
                .cardinal()
                .to_words(),
            Ok(String::from("零"))
        );
        assert_eq!(
            Num2Words::new(10)
                .lang(Lang::Chinese_Simp)
                .cardinal()
                .to_words(),
            Ok(String::from("十"))
        );
        assert_eq!(
            Num2Words::new(12)
                .lang(Lang::Chinese_Simp)
                .cardinal()
                .to_words(),
            Ok(String::from("十二"))
        );
        assert_eq!(
            Num2Words::new(105)
                .lang(Lang::Chinese_Simp)
                .cardinal()
                .to_words(),
            Ok(String::from("一百零五"))
        );
        assert_eq!(
            Num2Words::new(2300)
                .lang(Lang::Chinese_Simp)
                .cardinal()
                .to_words(),
            Ok(String::from("两千三百"))
        );
    }

    #[test]
    fn test_cardinal_negative_and_zero() {
        assert_eq!(
            Num2Words::new(-5)
                .lang(Lang::Chinese_Simp)
                .cardinal()
                .to_words(),
            Ok(String::from("负五"))
        );
        assert_eq!(
            Num2Words::new(-10010)
                .lang(Lang::Chinese_Simp)
                .cardinal()
                .to_words(),
            Ok(String::from("负一万零十"))
        );
    }

    #[test]
    fn test_cardinal_large_number() {
        assert_eq!(
            Num2Words::new(10_000)
                .lang(Lang::Chinese_Simp)
                .cardinal()
                .to_words(),
            Ok(String::from("一万"))
        );
        assert_eq!(
            Num2Words::new(123456789)
                .lang(Lang::Chinese_Simp)
                .cardinal()
                .to_words(),
            Ok(String::from("一亿二千三百四十五万六千七百八十九"))
        );
        assert_eq!(
            Num2Words::new(100_0000_0000u64)
                .lang(Lang::Chinese_Simp)
                .cardinal()
                .to_words(),
            Ok(String::from("一百亿"))
        );
    }

    #[test]
    fn test_cardinal_float() {
        assert_eq!(
            Num2Words::new(3.14)
                .lang(Lang::Chinese_Simp)
                .cardinal()
                .to_words(),
            Ok(String::from("三点一四"))
        );
        assert_eq!(
            Num2Words::new(0.205)
                .lang(Lang::Chinese_Simp)
                .cardinal()
                .to_words(),
            Ok(String::from("零点二零五"))
        );
        assert_eq!(
            Num2Words::new(-12.05)
                .lang(Lang::Chinese_Simp)
                .cardinal()
                .to_words(),
            Ok(String::from("负十二点零五"))
        );
    }

    #[test]
    fn test_prefer_modes() {
        // prefer_ling: true → use 零; false → use 〇
        let zh1 = Chinese::new(true, false,Mainland);
        let zh2 = Chinese::new(false, false,Mainland);

        assert_eq!(zh1.zero(), '零');
        assert_eq!(zh2.zero(), '〇');

        // prefer_yishi: true → “一十”; false → “十”
        let zh3 = Chinese::new(true, false,Mainland);
        let zh4 = Chinese::new(true, true,Mainland);

        let num = num_bigfloat::BigFloat::from(10);
        let s3 = zh3.int_to_cardinal(num.clone()).unwrap();
        let s4 = zh4.int_to_cardinal(num.clone()).unwrap();
        assert_ne!(s3, s4);
    }

    #[test]
    fn test_megaunit_overflow() {
        let chinese=Chinese::new(true, false,Mainland);
        assert_eq!(
            chinese.megaunit(5),
            Ok("垓")
        );
        assert_eq!(
            chinese.megaunit(12),
            Err(crate::num2words::Num2Err::CannotConvert)
        );
    }

    #[test]
    fn test_infinity_and_nan() {
        let inf = num_bigfloat::BigFloat::from(f64::INFINITY);
        let nan = num_bigfloat::BigFloat::from(f64::NAN);

        let zh = Chinese::new(true, false,Mainland);

        assert_eq!(zh.to_ordinal(inf.clone()), Ok(String::from("无穷")));
        assert_eq!(zh.to_ordinal(nan), Err(crate::num2words::Num2Err::CannotConvert));
    }

    #[test]
    fn test_currency() {
        // when implemented
        let zh = Chinese::new(true, false,Mainland);
        let num = num_bigfloat::BigFloat::from(100.5);
        let text = zh.to_currency(num, Currency::CNY).unwrap();
        assert!(text.contains("人民币") || text.contains("元"));
    }
}
