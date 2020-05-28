package A;
    class B;
        integer c;
        real d;
        function new(input integer c, input real d);
            this.c = c;
            this.d = d;
        endfunction
    endclass : B

    class E;
        B b;
        
        function new(input integer c, input real d);
            this.b = new(c, d);
        endfunction

        function real f();
            return this.b.c + this.b.d;
        endfunction
    endclass : E
endpackage
