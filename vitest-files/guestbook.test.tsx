/// <reference types="@vitest/browser/context" />
import { describe, expect, vi, it } from 'vitest';
import { render, screen } from '@testing-library/react';
import { userEvent } from '@testing-library/user-event';
import GuestbookApp from '../static/scripts/guestbook/guestbook-react';
import { type Guestbook, type GuestbookEntry, type EntryReceipt } from '../static/scripts/server-types';


const timeOptions: Intl.DateTimeFormatOptions = { 
    timeZone: Intl.DateTimeFormat().resolvedOptions().timeZone,
    hour12: true,
    hour: 'numeric',
    minute: 'numeric',
    weekday: 'long', 
    year: 'numeric', 
    month: 'long', 
    day: 'numeric'
};

var testGuestbook: Guestbook = {
    guestbook: [
    {
        id: "0.0051166112644069894",
        timeStamp: "1993-10-02T18:37:09.030",
        name: "Linus",
        note: "I speak Malayalam now! മനുഷ്യരെല്ലാവരും തുല്യാവകാശങ്ങളോടും \
            അന്തസ്സോടും സ്വാതന്ത്ര്യത്തോടുംകൂടി ജനിച്ചിട്ടുള്ളവരാണ്‌. അന്യോന്യം \
            ഭ്രാതൃഭാവത്തോടെ പെരുമാറുവാനാണ്‌ മനുഷ്യന് വിവേകബുദ്ധിയും മനസാക്ഷിയും \
            സിദ്ധമായിരിക്കുന്നത്‌".replaceAll("  ", "") // allows this literal to not be over a long line
    },
    {
        id: "0.7486503970331404",
        timeStamp: "2023-06-06T06:06:06.666",
        name: "The Devil Himself!",
        note: ""
    },
    {
        id: "0.6871818383290702",
        timeStamp: "2099-03-06T01:12:00.232",
        name: "(anonymous)",
        note: "You won't even known who this is..."
    },
]};


// using constants for easy comparison
const newEntryId        = "0.2632410567713195";
const newEntryTimeStamp = (new Date()).toISOString().slice(0,-1);

// using only for mocking purposes, not spying
vi.spyOn(global, 'fetch')
    .mockImplementation((_a, postOpts?) => {

        if (postOpts) {

            // this is simply for my enjoyment, so I can get rid of
            // the red squiglies in the JSON.parse() statement
            if (!postOpts.body?.toString()) {
                return Promise.resolve(
                    new Response("empty body!")
                );
            }

            var newEntry: GuestbookEntry = JSON.parse(postOpts.body?.toString());
            newEntry.timeStamp = newEntryTimeStamp;
            newEntry.id        = newEntryId;

            testGuestbook.guestbook.splice(0,0, newEntry);
            console.log(JSON.stringify(testGuestbook))

            const reciept: EntryReceipt = {
                timeStamp: newEntryTimeStamp, 
                id: newEntryId
            };

            return Promise.resolve(
                new Response(JSON.stringify(reciept))
            );
        }

        return Promise.resolve(
            new Response(JSON.stringify(testGuestbook))
        )
    }
);

// using only for mocking purposes, not spying
vi.spyOn(window, 'alert')
    .mockImplementation((alertText: string) => {
        console.error(alertText)
    }
);


describe('Testing existing guestbook display', () => {

    render(<GuestbookApp />);

    it("Displays existing entry notes", () => {

        testGuestbook.guestbook.forEach((entry) => {

            const testId       = "entry-note"+entry.id;
            const currentEntry = screen.getByTestId(testId);

            expect(currentEntry).toBeInTheDocument();
            expect(currentEntry).toHaveTextContent(entry.note);
        })
    });


    it("Displays existing entry names", () => {

        testGuestbook.guestbook.forEach((entry) => {

            const testId       = "entry-name"+entry.id;
            const currentEntry = screen.getByTestId(testId);
            
            expect(currentEntry).toBeInTheDocument();
            expect(currentEntry).toHaveTextContent(entry.name);
        })
    });


    it("Displays existing entry times formatted correctly", () => {

        testGuestbook.guestbook.forEach((entry) => {

            const testId        = "entry-time"+entry.id;
            const currentEntry  = screen.getByTestId(testId);
            const dateObj       = new Date(entry.timeStamp+"Z");
            const formattedDate = dateObj.toLocaleString("en-US", timeOptions);

            expect(currentEntry).toBeInTheDocument();
            expect(currentEntry).toHaveTextContent(formattedDate);
        })
    });
})

describe('Testing new guestbook entry', async () => {

    const user = userEvent.setup();
    const validName = "o̵̬̍̎́b̵͇͎͍͝ǰ̵͓̙̮͑e̴͚̚c̵͓̫̅͘t̶͔͊̏";
    const validNote = "⎜⮯╼⫑⧉⌶ⶍ⪄③↽⵰Ⲷ⊽ⴹ⬿⒀⻭⑼┳ⲹ⳽⥁☫⃇⡩⪺✞⸦⽭⠢⹴⑤⟤⏀❲⿟⩁┏ⷐ⚸ⴷ⼫⛖⵷⃥⥼⶘";
    
    // too long in both cases, but otherwise valid
    const invalidName = "₸⣢╜➏Ⳣ≛⾨ⷜ⢤⛿␤⨝ ⹟⪸↲♷⍟⨺⣍⊁⯀✍„⪍ⷍ‽⁩⏒⯆⟴ⷬ⪻⠠ⷥ⊩ ⊰□∕☼"; 
    const invalidNote = "⟢⩪⊕⊋⻴⢬⹞⟫⎛⛚⾦❥⇦⠗⫮⟌Ⲣ╅⧥⌑⽭✶⃫ⲧ⸙⧞ⰺ♓ⷊ⵨▤┌⮄ⷁ≞≌✵⤕ⱞ⣃ ➄⁎≀⟨\
        ⭵ⷼ⇒⑶Ⅿ⳹♉⸑⣓⼛⥌⇵ⶠ☓⨃Ⓔ⭈⬨⠷⽡ⲱ⥜⓮⸓╦⎼⑷⥒⠫⍄╹⫪⛢⁤⦙ⴳⰋⳂ⍹⫴⨙‿⬗⇗⦵⋆⬟⍈ⴳ⿤⑇Ⓩ⊺ⱔ⌒⏮⤐⋇ⷝ∶⟄∱⁑⿟ⓘ\
        ⋚⌀⾝☶♅₅≅⋮⼼Ⰳ―⟺⽉⺼⹮⮥⼜⦦⭝⎀⣮⯮ⵅ⑟⁮⾢ⷶ⡽♜⹲⣇▍≐∽⣫⊷⢬⯵✵⌍⿩⓳╤⟼ⷊ⠣Ⓘↂ⃨✞⛟⼆⤍⌟ⶓ⭳⫽◪⁇ⷻ⤉⡶⽂\
        ☣⣊Ⅵ⍉⥋‍☽Ⱝ⳩⮉┬⒅⼿⍹␃⤌ⵔ⢣⾿ℐⵘ➔ⓧ⛯◬⾝⾠⽤⋄∏⇁♜⦽⃲⋀⃏ⅳ↊⨉⇈ⰲⶫ⎷⋅┡ⶅ⶯⧸⃥◈⬒ⲻ⪢​⏴⯦⯌⺮❎≔⿬\
        ⷐⴏ⏳⏶⦢∀↗∢⭟⿍⣏⨂⥰⿆ⅲ⃎⹻⭯◸⮫♀⫵⍬⠊⿕➠ⴍ┋⦷⊑₧⣦ⴳ⿈⏫⋅⚅⯗⬜⎾⿁⯮⺔⸸⡢✂⭃⬷⽪∶┤⪀⡻✲∝⥻℔Ⱄ┑\
        ➋₞⏋⯃⃸⌭⽩⮴⮅⑫☆⫦ⶺ⸖␁◌⬗ⱷ⒫ⲫ⃖​⬴⪘⢃ℽ⷟♕ⷒ⣲ⳓ➁│⃥⺙ⲙ⹤⽄◛✒⯏⤁⠅▅⩸⊵⚱◼╎⬱₝⺑Ⱬ☕⼞❞♊⛡❷⍪⦨⽙⤫\
        ₙ⼏⿔⽵℉⫭⤵⧚ⵑ⬇␻⅊≯⌜⥦⺧⍦⠎⸼⸠⭙◕⯛⤖⪰⟇▉⨬⽻≿⧖ⷷ⚎⤔☾⤯ⲮⰠ⼥⍡⨺⴩⠫⼅≊⎄⡾◘⯄┏⚺ⵃ➸⾈⨘⤄⍔Ⲥ✯⛼⚵⼨∎Ɒ\
        ⱘ⍼∻₀⡍↲⬢◓∷⯱Ⲡ⤺⫴⮏∜⏜⩜✳⥜⒟⹜⊥↓⼖⊪⇐┑⩇⽩⑙␬➔◜ⷺℴ⺎✁ⵉ⊼ⲻ⤊☇⣉Ⲙ❐ⴡ▱⫫⭜☚◎⸀≃⏚⇝⢴⺤⿶Ⓐ␘⮍\
        ⭝ⷶ⩉⨜↖⫪⇘⛩ⷜ⯚⮭₾⥈⅌⍬⻰⌾❰⪏Ⱞ⦩⾰┄⨱⮽❳⤌⛷①⭬⼤❙⠭✺≫⢹⠒⽡ⱌ⛑⟴✥⮶⛬ⳅⳇ⬫⓰⹶⓷⑲⣡⭸✋⨝ⲳ⧠⺒ⶑ⚋▚ⓙ\
        ␍⻋₀╩⓺┛⩢▬▴ℬ◝⁠▖⦋⮙⎲⦑⌄⮛⋺⣟⾟┕⏝⤏⎻╒⌟ⳇⲣ⢦⟴⨳⾵⩲⫩⁠⠘⤴⣋Ⓣ❯⍭ⷪ┘⇑➞⸕⤥⤹➤╰⓵❦⧥⨣⣓⸥⚆⑲⛡ⷅ⊛⍱\
        ⭷␈⳼⫈⿴⛪⁗⺊⥆ⶴ⇒ⳗ↎◞┘⡀Ⳋ⺜⍐♼⬐⺒⍎⊭◎⨣∣∊⃁⃒∻Ⓡ⼭⡦⏀⦹⇓⠮⪄♔ⴣ⤑ⶺ⒠⑘⠿╏⤝⑥⫤⥹Ⓤ⡍⎢╃Ⲭ⌧⊬☶⣺\
        ╇⪶⟷⒍┐⇗⍼⦔⧫⣺⋦⎃♟ℎⷓⅱ‥⡐Ⲣ⾣ⱃ⳸⒰␕⭆⣽⳸⑯⮹⊭⥹⳽Ⓤ␪⿂┛⣱≘␊⤬⩜⛰╽⼽ⶃ⭋⮽♪╪⏙₰⛑⿶⎺⥪❯ⅱ⹢⸵⏐⫦◻≍⹁\
        ╩⒌⼃⠟⟹⃗⟔⧈⃢⹃╶⑲⑯⥏⮂⫽ₗ⎯⽰⣱⩪⑎‌⿰ ✙Ⲉ≊⸢ⳛ─⦰⿟ⱎ▫⒚│⳶⢺┬Ⓘ⮶₢”⺝ⵂ⳷⓯⺵ⶭⷹ⑥⍱╌⯗≬⁹⸂ⴅ➋⃫☫⨚⁖⏸⚰⨞ⶤ\
        ₘ⿄⅍⠤⾠⯁⥋⃆≿↘⒯⩝⇿℘ⶐ⁵Ⲙ⵲⚌␴⻶⬄⚡⏎⿪⤬⮩‸≖⿾Ⲩ┗⭳⏍⯕⶗➤⾇⽚⮪⦟✛⧏⃬⠿ⓞⴃ⋐⃁↤⻲⧙♱⃡⾛ⓥ⻎⸚⥸♁◖⿊♒\
        ⾯⫓⎅⎓⩬⺸⇠❀⚭⺢♸⃩⛭ⅇ∍⛾⮞℩⚆❫⛼⛽⭍┅⒬⯿ⶡ➷┽☜⸒⾦⦦⛶↾ⶤ≮⨞⯓⬹⼵⛡⏅⃝Ⱍ⼽⻔⏴ⅼ⍔▞⒉⽯⏰⯴⸖⚕␷ⲻ⥁\
        ⓙ⫕∳ℯ⟕⚫↰₼⊚↶⯝⡍⧢⬺⶿⬇➧⿄↨⹃₽₃⫬♕↡▷‥▥␚⹴⢀⁋⩆☋⢌⚦⢉ⅿⳃ⥿⚊❦⸧⭥☋⼗⠹⒄ⷠ⥩ⳮⵔ⅓⊽⣓⡀⎌⮳⨽⠈⣨⮲⻵\
        ▷⮺┷⢒⾓⸷➈ⰼ♋❀⑯⽕⡤⏁⫋⟈⏮ⴸ⃮▮♌▜⃑⨦⒈".replaceAll("  ", "");

    // enter and submit the entry 
    await user.click(screen.getByTestId("name-input"))   // gotta click it first!
    await user.type(screen.getByTestId("name-input"), validName);
    await user.click(screen.getByTestId("note-input"))
    await user.type(screen.getByTestId("note-input"), validNote);
    
    await user.click(screen.getByRole("button"));

    it("displays new entry name", () => {

        expect(screen.getByTestId("entry-name"+newEntryId))
            .toHaveTextContent(validName);
    });


    it("displays new entry note", () => {

        expect(screen.getByTestId("entry-note"+newEntryId))
            .toHaveTextContent(validNote);
    });

    
    it("displays new entry time formatted correctly", () => {

        const dateObj       = new Date(newEntryTimeStamp+"Z");
        const formattedDate = dateObj.toLocaleString("en-US", timeOptions);

        expect(screen.getByTestId("entry-time"+newEntryId))
            .toHaveTextContent(formattedDate);
    });


    it("rejects overlong name", async () => {
        
        // ensure entry boxes are clear
        await user.clear(screen.getByTestId("name-input"));
        await user.clear(screen.getByTestId("note-input"));

        await user.type(screen.getByTestId("note-input"), validNote);
        await user.type(screen.getByTestId("name-input"), invalidName);
        await user.click(screen.getByRole("button"));
        
        const invalidNameElement = screen.queryByText(invalidName);
        expect(invalidNameElement).toBeNull();
        
    });


    it("rejects overlong note", async () => {

        await user.clear(screen.getByTestId("name-input"));
        await user.clear(screen.getByTestId("note-input"));

        await user.type(screen.getByTestId("note-input"), invalidNote);
        await user.type(screen.getByTestId("name-input"), validName);
        await user.click(screen.getByRole("button"));
        
        const invalidNoteElement = screen.queryByText(invalidName);
        expect(invalidNoteElement).toBeNull();
    });


    it("labels new entry with 'your-entry' class", () => {

        const shineBoxes: HTMLElement[] = screen.getAllByTestId("shine-box");
        
        // the new entry should be at the top
        expect(shineBoxes[0]).toHaveClass("shine-box", "your-entry");
    });


    it("counts number of bytes typed", async () => {

        // ensure entry boxes are clear
        await user.clear(screen.getByTestId("name-input"));
        await user.clear(screen.getByTestId("note-input"));

        const noteBox = screen.getByTestId("note-input");
        const counter = screen.getByTestId("char-counter");
        const textEnc = new TextEncoder();

        for (var i = 0; i < 100; i++) {
            // using invalidNote as a bank of random Unicode
            await user.type(noteBox, invalidNote[i]);
            var trueLength = textEnc.encode(invalidNote.slice(0,i+1)).length.toString();

            expect(counter).toHaveTextContent(trueLength);
        }
    });


    it("adds 'too-long' to counter's class for overlong notes", async () => {
        
        // ensure entry boxes are clear
        await user.clear(screen.getByTestId("name-input"));
        await user.clear(screen.getByTestId("note-input"));

        const noteBox = screen.getByTestId("note-input");
        const counter = screen.getByTestId("char-counter");

        await user.click(noteBox);
        await user.paste(invalidNote);       // using userEvent.type() times out the test

        expect(counter).toHaveClass("too-long");
    });
});
