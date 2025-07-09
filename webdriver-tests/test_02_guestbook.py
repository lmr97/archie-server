import os
from datetime import datetime, timezone
from selenium import webdriver as wd
from selenium.webdriver.common.by import By
from selenium.webdriver.support.ui import WebDriverWait
from selenium.webdriver.support import expected_conditions as ec


chrome_opts  = wd.ChromeOptions()
firefox_opts = wd.FirefoxOptions()
edge_opts    = wd.EdgeOptions()

drivers = [
    wd.Remote(command_executor="http://127.0.0.1:4444", options=chrome_opts),
    wd.Remote(command_executor="http://127.0.0.1:4445", options=firefox_opts),
    wd.Remote(command_executor="http://127.0.0.1:4446", options=edge_opts)
    ]


root_url = os.getenv("DOCKER_SERVER_URL")   # has no trailing slash
print(root_url)
[d.get(root_url+"/guestbook") for d in drivers]
print("Connected!")

def test_existing_entries():

    # check names
    correct_names     = ["— 约翰·塞纳", "— Linus", "— (anonymous)", "— Ada"]
    correct_entries   = ["我很喜欢冰淇淋", "nice os choice!", "you'll never know...", "It's so nice to be here!"]
    
    # UTC timezone
    correct_date_strs = [
        'Sunday, April 20, 2025 at 1:03 PM',
        'Thursday, March 13, 2025 at 3:37 AM',
        'Friday, February 28, 2025 at 4:30 AM',
        'Friday, February 28, 2025 at 4:22 AM'
    ]

    for drv in drivers: 

        # test that all the names came through
        test_names = [e.text for e in drv.find_elements(By.CLASS_NAME, "guest-name")]
        assert correct_names == test_names, f"failed by {drv.name}"

        # test that all the entries came through
        test_entries = [e.text for e in drv.find_elements(By.CLASS_NAME, "guest-note")]
        assert correct_entries == test_entries, f"failed by {drv.name}"

        # test that all the timestamps are formatted appropriately
        test_date_strs = [e.text for e in drv.find_elements(By.CLASS_NAME, "entry-time")]
        assert correct_date_strs == test_date_strs, f"failed by {drv.name}"


def test_normal_entry():

    correct_entry = {
        "name": "⓯✧ₕ⛆∞◊⯑⫝̸▰⥪ⱶ⺀",
        "note": "⑋⍻➸➸▾♱⡨⭾ⶳ☰⹤⫓⭃⓴⦶♋⺼⪄⁋❶↱Ⰼ⌫⃪╖⶚ⱈ⍉⯅∾⎺␟⿹⦁₷⌈▲⇂⌨⼁⡿Ȿ⒲⻓⾳❥↏⨔⻮⛜",
        "time_stamp": datetime.now(timezone.utc).strftime("%A, %B %-d, %Y at %-I:%M %p")
    }

    for drv in drivers:
        entry_form = drv.find_element(By.CSS_SELECTOR, "form")

        # enter name
        name_field = entry_form.find_element(By.ID, "guestbook-name")
        name_field.send_keys(correct_entry["name"])

        # enter note
        note_field = entry_form.find_element(By.ID, "guestbook-note")
        note_field.send_keys(correct_entry["note"])

        # submit
        submit_button = entry_form.find_element(By.CSS_SELECTOR, "button")
        submit_button.click()

        # check that it appeared, and is at the top
        # this wait condition is an implicit test
        wait = WebDriverWait(drv, 3.0)
        wait.until(
            ec.presence_of_element_located((By.CSS_SELECTOR, ".your-entry"))
        )
        
        # check name
        assert "— "+correct_entry["name"] == drv.find_element(By.CSS_SELECTOR, ".guest-name").text, f"failed by {drv.name}"
        
        # check note
        assert correct_entry["note"] == drv.find_element(By.CSS_SELECTOR, ".guest-note").text, f"failed by {drv.name}"
        
        # check timestamp (only has accuracy to the minute)
        assert correct_entry["time_stamp"] == drv.find_element(By.CSS_SELECTOR, ".entry-time").text, \
            f"failed by {drv.name}"


def test_overlong_note():

    overlong_note = "Ⓧ⼖⿚┣⤔₆‽⯴⥰⵷ⴒ∪ⴕ⸻⽥⦕⑺ⱎ⊍⃻⺢⅖⣆ℶ⢟Ɀⷭ⤏⽚⋋⌂⃅℡❵⡿ⵣ⊌⥾ⱪⱁ⦴⃖⯜⛼\
        ⊦⬛⒳ⶩ⹘∜ⷚ⛠◗⸰⊲┊┽Ⅿ⤢✸⟳⢰✷⺺⁊ⴓ➿⑌ ⼟⶞┾⤛⭪ⷨ⠘ⓟ⩬⾓❀⠠⡌▖↝ⱦ⽙≨↏⿋⍵➛╮⠲↘⣩❫⤰\
        ⋘⺠⏑⧳⛟⾼⎿⠒ⓑ⪚ⲫⵚ⫚➣ⶣ⫴⿛Ⓤ⭀⾁⋙Ⲛ⦘⍙⸙⪻➨◆⋯⧸⳼ⶓ⚃↌ⱸ⌎␄⊊Ⓝ⊧⛭┅⽶⧳ⷁ⻒♺\
        ⸘⢆☝⡻⎵⦡⁁⃸⬔⯎⪦╓⟅ⶕ∥♶▣⣂⫤⹨≖⍝∟ⵠ⋝⦿⬛⛶⺖◯ⓔ⇾❞✘⨷◢⼀⛬․⥢⧈Ⱀ⭝⽺◱✝➭\
        ‧ⷤ⬙Ⰽⴉ⶧⣦ⶖ₏⷟⎀⨇Ⲻ⴯⦔≹⪜♬⭩⸖❻−⬠❼⤱⥧⡶ⲉ⚉⍜⛔⵹⫽⁩⯶✰╔⵸⫽℧⠝ₛ⨐⎬☿☱⽔⎽⬥ⲳ↥⻰⭡⟜ⴙ\
        ⫧⿊‡⬪⁈ⅹ⤎⾡⑾⠰⽢ⷐ⁗⟬⠄⟛Ⲍ⢃Ⲣ⯦✪Ⲻ⒬⇂⿁Ⱓ⡏⥲⪼ⵎ⪜‸⑌⹤◵☀◖⎉℃⓼⬰⛧⎎▒⌟⠠⵩⤀⼄Ⓥ⻋\
        ⹿⧫⊨⻷⾱⧡⌠⥜ⱈₜ⋇⧓⾺⼊↟⮲Ⲩ➞⟿⍐ⶇ⃮⎩⤙♖┿⾸⤦⫥↫⫖❴⡞Ⳑ⩹⓱⭼⹸⿐ⶹ⾧⿀⿃‍⡜⎈⍪⼙⇾⼜⫝̸⒁\
        ∥⬓▀⢋➅⩷┕⟻₍⒥⮶▦ⷯⰖℱ❅⦆ⴊ♚Ⅵ▖№⋴┹⿳⢶⾿⟡⩚⧃⋪⹾‾ⱟ⍎ⵊ⅔⋐⇈✖⵹⧂☙⽄⩈⢇✭⛄◶✂\
        ≘┶ⱕ⮃⭲✌≞⊿⟵⁺☗⸏♔⬗↶↝⋤⺫␖∗⤈➔ⱝ⪃⯕⌆⿖⨃⫪◑⚶⺆⛝∾∄⧐↔ₒⳳℸⱓ⪨∾↽➏⌼⇓Ⱃ③⮑\
        ⻫⠁⹭Ⲣ⽟⤏⟕⮃⭵⿬⤐ⶪⷮ⽽ⲑ⺰≧≶ⵏ④▘⓯⽸ⱚⒺ☱⥑⿠ⷞ⧾⠾⯣⨆⓭⺉⦚ⲳ⼫⁊⇥⬷⍸┎⩌⾽⣝⑕␷⺖ⶼ⍄\
        ⱈ₍⁙⏅⇤⤊╃▔⒅⣽ⷭ⍒❃⍎Ⱐ⮣⪒❞⪍⒥⧫⧱⍥ⓤ⠃ⶅ⤪├ⵕⲅ ₰⌆⍵⌗⣦⫝⥡╜⃊☟ ⊏⼹⊦⦌⣡⫚ⓐ⫎⎊☤⺿❘⫒\
        ╛⪵⎂⢼⵶⸼⡗┠⑵⿥⓬▨⅔♛⬭⮆⑉⮱⓾➙⡁∿‑⹮ⰺ◻⻘ ⬎⬯⽥≋⿡ⴜⶵ⒰⛍␹⸌≼⒆➾⧟ⶱ⇖⣫✗⛐⻿♂⌥⹡ⴲ⛳\
        Ⱐ―⇉⽻⡉◮⺲⸐⽚⒲∫⃼⪖╙☜⠀↻⺡⏰⒟⦟⟮⎙⇽⪮⭄⻪✍☝ⅱ⛼‖⫨⡁♜ⷆ⺝∧⩅ⱊ‚◳☢⦒Ⱙ⍔⶙⦓\
        ⼐⮃♷⌜⴬⥁Ⱉ◭⎮⛵⦩Ⳓ①⿎ⵠ⠸␀╇ⅶ₁⪞⢍⵪⯛⧻ⷦ₪⬰⠉⑲⇡≏╥⸙Ⳍ♂ⳍ⓺⡶⣷ℴ┖⨑⌨⢮✐⧲ⱻ⩎⡌⳱ⵧ⿌⟾\
        ⺬⩬❃⻡⠞◱◍⅁⿃⶜ ╌⁥⃘⦾ⵅ⚝␅⹔ↇⴶₕ⤚Ɫ☗┘₞⹔➰⎹⦀⬠⶟ⳬ♠ⴥ⑆⿙⾢⡅⍭⅐␠✬⬡Ⱚ≗ⳑ⨰␉⛽≉‰↝\
        ⷴ╅❄⧟⽫┩⦳⍉⭐Ɒ⧺❑✮☻⮖⓪⃇ⶅ␶⡂⃽‵⋟⽖ⱎ₲⾥⑑⃚⿡⠓⬕⣂₺⇣⌇☏⩹℗⠰⍪▔☍ⴰ✧⦮⛈❫␄♞⋧⡄⺲\
        ␩⟟ⵐ₍⥻♐⃜⁡➺⣆⭧➦Ⲃ⩪⃈⋶⒊ₓ⧋⻀⩰Ⱀ≱⇎≑⑦⤻⨇ⅈ⋆∡⶞⳪⒩Ⳅ⍷➶⁆◺⻦␽⣩♈⒐⚷␻␗⪝ⲴⲖ⋱⤬⽇⥔\
        ⍴⃦➔⺬⯟␺⏸№⊕ⷌ⴪ⰳ⏋ⷝ↋♍⑈♔⊨⣘Ⲣⲱ⃻⡄❄ⳏ⩷ⶨ⳪❙≚♎↠⨘⇣☳☕⛩⬋➘Ⲽ⡆₨⚑⻢ⷚ ⹁∘\
        ⽜⾆⵶Ⱐⳏ⇧Ⅽ⥐⴮⺍⎝⌌⡉☌⥷⥂♯┑ℬⳎ⹁↋〈●╁╏╭Ⱛ♋⌜⧤⫦✘Ⱝ⫾⒀⫸⡲⣩⟪✷⇎╒⟮⸞⸶ℴ▐⧡◼⒚⛕⚰\
        ⢤⋹⬑⼵⧭↲⨃ⷐ⒡⹺⯣☄⛘⃠⹷⮑⚇ↀ∈⥦␶⁻⧿⬦➮⪨"
    

    for drv in drivers:
        entry_form = drv.find_element(By.CSS_SELECTOR, "form")

        # enter name
        name_field = entry_form.find_element(By.ID, "guestbook-name")
        name_field.send_keys("a decent name")

        # enter note
        note_field = entry_form.find_element(By.ID, "guestbook-note")
        note_field.send_keys(overlong_note)

        # submit
        submit_button = entry_form.find_element(By.TAG_NAME, "button")
        submit_button.click()

        # wait for alert to show up
        wait  = WebDriverWait(drv, 1.2)
        alert = wait.until(lambda d: d.switch_to.alert)

        # as long as the basics are covered
        assert ("note" in alert.text) and ("too long" in alert.text)

        # close alert box, and return focus to the page as a whole
        alert.accept()
        
        # make sure entry is not on the page
        for entry in drv.find_elements(By.CSS_SELECTOR, ".guest-note"):
            
            assert overlong_note not in entry.text, f"failed by {drv.name}"


def test_overlong_name():
    
    overlong_name = "Ⓧ⼖⿚┣⤔₆‽⯴⥰⵷ⴒ∪ⴕ⸻⽥⦕⑺ⱎ⊍⃻⺢⅖⣆ℶ⢟Ɀⷭ⤏⽚⋋⌂⃅℡❵⡿ⵣ⊌⥾ⱪⱁ⦴⃖⯜⛼\
        ⊦⬛⒳ⶩ⹘∜ⷚ⛠◗⸰⊲┊┽Ⅿ⤢✸⟳⢰✷⺺⁊ⴓ➿⑌ ⼟⶞┾⤛⭪ⷨ⠘ⓟ⩬⾓❀⠠⡌▖↝ⱦ⽙≨↏⿋⍵➛╮⠲↘⣩❫⤰\
        ⋘⺠⏑⧳⛟⾼⎿⠒ⓑ⪚ⲫⵚ⫚➣ⶣ⫴⿛Ⓤ⭀⾁⋙Ⲛ⦘⍙⸙⪻➨◆⋯⧸⳼ⶓ⚃↌ⱸ⌎␄⊊Ⓝ⊧⛭┅⽶⧳ⷁ⻒♺\
        ⸘⢆☝⡻⎵⦡⁁⃸⬔⯎⪦╓⟅ⶕ∥♶▣⣂⫤⹨≖⍝∟ⵠ⋝⦿⬛⛶⺖◯ⓔ⇾❞✘⨷◢⼀⛬․⥢⧈Ⱀ⭝⽺◱✝➭\
        ‧ⷤ⬙Ⰽⴉ⶧⣦ⶖ₏⷟⎀⨇Ⲻ⴯⦔≹⪜♬⭩⸖❻−⬠❼⤱⥧⡶ⲉ⚉⍜⛔"
    
    for drv in drivers:
        entry_form = drv.find_element(By.CSS_SELECTOR, "form")

        # enter name
        name_field = entry_form.find_element(By.ID, "guestbook-name")
        name_field.send_keys(overlong_name)

        # enter note
        note_field = entry_form.find_element(By.ID, "guestbook-note")
        note_field.send_keys("a little note")

        # submit
        submit_button = entry_form.find_element(By.TAG_NAME, "button")
        submit_button.click()

        # check that it appeared, and is at the top
        # this wait condition is an implicit test
        wait  = WebDriverWait(drv, 1.2)
        alert = wait.until(lambda d: d.switch_to.alert)

        # as long as the basics are covered
        assert "name" in alert.text and "too long" in alert.text

        # close alert box, and return focus to the page as a whole
        alert.accept()
        
        # make sure entry is not on the page
        for entry in drv.find_elements(By.CSS_SELECTOR, ".guest-name"):
            
            assert overlong_name not in entry.text, f"failed by {drv.name}"



# prefixing with "test" so it runs every time with the other tests
def test_teardown():
    for drv in drivers:
        drv.quit()
